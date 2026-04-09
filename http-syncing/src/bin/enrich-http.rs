use std::path::PathBuf;

use clap::Parser;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(about = "Measure HTTP clock offset for hosts with ICMP timestamp data")]
struct Args {
    /// Enriched CSV from enrich-ips-clockdiff
    input: PathBuf,
    /// Output comparison CSV
    output: PathBuf,
    /// Number of hosts to sample
    #[arg(long)]
    sample: usize,
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

fn main() {
    let args = Args::parse();

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(&args.input)
        .expect("failed to open input CSV");

    let mut candidates: Vec<InputRecord> = rdr
        .deserialize()
        .filter_map(|r| r.ok())
        .filter(|r: &InputRecord| r.had_date && r.clock_offset_ms.is_some())
        .collect();

    eprintln!(
        "found {} candidates with had_date=true and clock_offset_ms present",
        candidates.len()
    );

    let mut rng = rand::rng();
    candidates.shuffle(&mut rng);
    candidates.truncate(args.sample);

    eprintln!("measuring {} hosts", candidates.len());

    let mut wtr = csv::Writer::from_path(&args.output).expect("failed to open output CSV");

    for (i, rec) in candidates.iter().enumerate() {
        let url = format!("http://{}", rec.ip);
        eprintln!("[{}/{}] {}", i + 1, candidates.len(), rec.ip);

        match clocks::measure_host(&url) {
            Ok((http_rtt_us, http_clock_offset)) => {
                let out = OutputRecord {
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
                };
                wtr.serialize(&out).expect("failed to write row");
                wtr.flush().expect("failed to flush");
                eprintln!(
                    "  icmp_offset={}ms http_offset={}ms",
                    rec.clock_offset_ms.unwrap(),
                    http_clock_offset.num_milliseconds()
                );
            }
            Err(e) => {
                eprintln!("  failed, skipping: {e}");
            }
        }
    }
}
