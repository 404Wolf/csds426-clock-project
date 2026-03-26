use std::fs::{File, OpenOptions};
use std::net::IpAddr;
use std::path::PathBuf;

use clap::Parser;
use icmp_clocksync::shared;
use icmp_clocksync::{EnrichedRecord, IcmpTimestampRecord, get_latest_batch, iter_icmp_csv};
use itertools::Itertools;
use rayon::prelude::*;

const BATCH_SIZE: usize = 100;

#[derive(Parser)]
struct Args {
    /// Input CSV file
    input: PathBuf,
    /// Output CSV file
    output: PathBuf,
    /// Path to GeoLite2-City.mmdb file
    #[arg(long, default_value = "GeoLite2-City.mmdb")]
    mmdb: PathBuf,
    /// Only process the first N rows
    #[arg(short = 'n', long)]
    limit: Option<usize>,
}

fn main() {
    let args = Args::parse();
    let input_path = args.input;
    let output_path = args.output;

    let geoip = maxminddb::Reader::open_readfile(&args.mmdb)
        .unwrap_or_else(|e| panic!("failed to open mmdb at {}: {e}", args.mmdb.display()));

    // Check where we left off
    let resume_after = get_latest_batch(&output_path);
    let skip_rows = resume_after.map_or(0, |b| (b as usize + 1) * BATCH_SIZE);

    if let Some(b) = resume_after {
        eprintln!("resuming after batch {b}, skipping {skip_rows} rows");
    }

    let file = File::open(&input_path).expect("failed to open input csv");
    let iter = iter_icmp_csv(file);

    // Open output in append mode so we don't clobber existing progress
    let append = resume_after.is_some();
    let out_file = if append {
        OpenOptions::new()
            .append(true)
            .open(&output_path)
            .expect("failed to open output csv for append")
    } else {
        File::create(&output_path).expect("failed to create output csv")
    };
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(!append)
        .from_writer(out_file);

    let start_batch = resume_after.map_or(0, |b| b + 1);

    let valid_rows = iter.filter_map(|r| match r {
        Ok(rec) => Some(rec),
        Err(e) => {
            eprintln!("skipping bad row: {e}");
            None
        }
    });

    let valid_rows: Box<dyn Iterator<Item = _>> = if let Some(n) = args.limit {
        Box::new(valid_rows.skip(skip_rows).take(n))
    } else {
        Box::new(valid_rows.skip(skip_rows))
    };

    let mut last_batch = start_batch;
    for (i, chunk) in valid_rows.chunks(BATCH_SIZE).into_iter().enumerate() {
        let batch_num = start_batch + i as u64;
        let batch: Vec<_> = chunk.collect();
        flush_batch(&mut wtr, &batch, batch_num, &geoip);
        last_batch = batch_num;
    }

    wtr.flush().expect("failed to flush output csv");
    eprintln!("done, last batch: {last_batch}");
}

fn flush_batch(
    wtr: &mut csv::Writer<File>,
    batch: &[IcmpTimestampRecord],
    batch_num: u64,
    geoip: &maxminddb::Reader<Vec<u8>>,
) {
    let resolved: Vec<EnrichedRecord> = batch
        .par_iter()
        .map(|record| {
            let ip = record.saddr.parse::<IpAddr>().ok();

            let hostname = ip.map(shared::resolve_hostname).unwrap_or_default();
            let (is_http, had_date) = ip.map(shared::probe_http).unwrap_or((false, false));
            let geo = ip
                .map(|ip| shared::lookup_geo(geoip, ip))
                .unwrap_or_default();

            let t1 = record.otime as i64;
            let t2 = record.rtime as i64;
            let t3 = record.ttime as i64;
            let t4 = t1 + record.rtt_ms as i64;
            let clock_offset_ms = ((t2 - t1) + (t3 - t4)) / 2;

            EnrichedRecord {
                batch_num,
                ip: record.saddr.clone(),
                hostname,
                rtt_ms: record.rtt_ms as f64,
                is_http,
                had_date,
                country: geo.country,
                city: geo.city,
                latitude: geo.latitude,
                longitude: geo.longitude,
                daddr: Some(record.daddr.clone()),
                otime: Some(record.otime),
                rtime: Some(record.rtime),
                ttime: Some(record.ttime),
                clock_offset_ms: Some(clock_offset_ms),
            }
        })
        .collect();

    for rec in &resolved {
        wtr.serialize(rec).expect("failed to write row");
    }
    wtr.flush().expect("failed to flush batch");

    eprintln!("batch {batch_num} done ({} rows)", resolved.len());
}
