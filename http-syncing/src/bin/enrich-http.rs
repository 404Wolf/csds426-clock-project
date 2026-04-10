use std::fs::{File, OpenOptions};
use std::path::PathBuf;

use clap::Parser;
use itertools::Itertools;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

const BATCH_SIZE: usize = 20;

#[derive(Parser)]
#[command(about = "Measure HTTP clock offset for hosts with ICMP timestamp data")]
struct Args {
    /// Enriched CSV from enrich-ips-clockdiff
    input: PathBuf,
    /// Output comparison CSV
    output: PathBuf,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct InputRecord {
    batch_num: u64,
    ip: String,
    hostname: String,
    rtt_ms: f64,
    is_http: bool,
    had_date: bool,
    country: String,
    city: String,
    latitude: f64,
    longitude: f64,
    daddr: Option<String>,
    otime: Option<u64>,
    rtime: Option<u64>,
    ttime: Option<u64>,
    clock_offset_ms: Option<i64>,
}

#[derive(Debug, Serialize)]
struct OutputRecord {
    batch_num: u64,
    ip: String,
    hostname: String,
    icmp_rtt_ms: f64,
    icmp_clock_offset_ms: i64,
    http_rtt_us: u64,
    http_clock_offset_ms: i64,
    country: String,
    city: String,
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, Deserialize)]
struct BatchOnly {
    batch_num: u64,
}

fn get_latest_batch(path: &PathBuf) -> Option<u64> {
    let file = File::open(path).ok()?;
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(std::io::BufReader::new(file));
    let mut max_batch = None;
    for rec in rdr.deserialize::<BatchOnly>().flatten() {
        max_batch = Some(max_batch.map_or(rec.batch_num, |m: u64| m.max(rec.batch_num)));
    }
    max_batch
}

fn main() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(50)
        .build_global()
        .unwrap();

    let args = Args::parse();

    let resume_after = get_latest_batch(&args.output);
    let skip_rows = resume_after.map_or(0, |b| (b as usize + 1) * BATCH_SIZE);

    if let Some(b) = resume_after {
        eprintln!("resuming after batch {b}, skipping {skip_rows} rows");
    }

    let in_file = File::open(&args.input).expect("failed to open input CSV");
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(std::io::BufReader::new(in_file));

    let candidates = rdr
        .deserialize::<InputRecord>()
        .filter_map(|r| r.ok())
        .filter(|r| r.had_date && r.clock_offset_ms.is_some())
        .skip(skip_rows);

    let append = resume_after.is_some();
    let out_file = if append {
        OpenOptions::new()
            .append(true)
            .open(&args.output)
            .expect("failed to open output CSV for append")
    } else {
        File::create(&args.output).expect("failed to create output CSV")
    };
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(!append)
        .from_writer(out_file);

    let start_batch = resume_after.map_or(0, |b| b + 1);

    for (i, chunk) in candidates.chunks(BATCH_SIZE).into_iter().enumerate() {
        let batch_num = start_batch + i as u64;
        let batch: Vec<InputRecord> = chunk.collect();

        eprintln!("batch {batch_num} ({} hosts)", batch.len());

        let results: Vec<Option<OutputRecord>> = batch
            .par_iter()
            .map(|rec| {
                let url = format!("http://{}", rec.ip);
                eprintln!("  measuring {}", rec.ip);
                match clocks::measure_host(&url) {
                    Ok((http_rtt_us, http_clock_offset)) => {
                        eprintln!(
                            "  {} icmp_offset={}ms http_offset={}ms",
                            rec.ip,
                            rec.clock_offset_ms.unwrap(),
                            http_clock_offset.num_milliseconds()
                        );
                        Some(OutputRecord {
                            batch_num,
                            ip: rec.ip.clone(),
                            hostname: rec.hostname.clone(),
                            icmp_rtt_ms: rec.rtt_ms,
                            icmp_clock_offset_ms: rec.clock_offset_ms.unwrap(),
                            http_rtt_us: http_rtt_us as u64,
                            http_clock_offset_ms: http_clock_offset.num_milliseconds(),
                            country: rec.country.clone(),
                            city: rec.city.clone(),
                            latitude: rec.latitude,
                            longitude: rec.longitude,
                        })
                    }
                    Err(e) => {
                        eprintln!("  {} failed, skipping: {e}", rec.ip);
                        None
                    }
                }
            })
            .collect();

        let succeeded = results.iter().filter(|r| r.is_some()).count();
        for out in results.into_iter().flatten() {
            wtr.serialize(&out).expect("failed to write row");
        }
        wtr.flush().expect("failed to flush");
        eprintln!("batch {batch_num} done ({succeeded}/{} succeeded)", batch.len());
    }
}
