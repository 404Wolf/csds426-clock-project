use std::fs::File;
use std::net::ToSocketAddrs;
use std::path::PathBuf;

use clap::Parser;
use log::{info, warn};
use rayon::prelude::*;

const SAMPLE_SIZE: usize = 20_000;

#[derive(Parser)]
#[command(about = "Sample Tranco list, resolve to IPs, write enrich-http input CSV")]
struct Args {
    /// Tranco CSV (rank,domain, no header)
    input: PathBuf,
    /// Output CSV for enrich-http
    output: PathBuf,
    /// Number of domains to sample evenly
    #[arg(long, default_value_t = SAMPLE_SIZE)]
    sample: usize,
    /// Rayon thread count for DNS resolution
    #[arg(long, default_value_t = 200)]
    threads: usize,
}

fn resolve(domain: &str) -> Option<String> {
    let addr = format!("{domain}:80");
    addr.to_socket_addrs()
        .ok()?
        .find(|a| a.is_ipv4())
        .map(|a| a.ip().to_string())
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    rayon::ThreadPoolBuilder::new()
        .num_threads(args.threads)
        .build_global()
        .unwrap();

    // Read all rows
    let in_file = File::open(&args.input).expect("failed to open input CSV");
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(std::io::BufReader::new(in_file));

    let rows: Vec<(String, String)> = rdr
        .records()
        .filter_map(|r| r.ok())
        .map(|r| (r[0].to_string(), r[1].to_string()))
        .collect();

    let total = rows.len();
    info!("{total} rows in input");

    assert!(
        total >= args.sample,
        "input has {total} rows, need {}",
        args.sample
    );

    // Evenly sample
    let step = total as f64 / args.sample as f64;
    let sampled: Vec<&(String, String)> = (0..args.sample)
        .map(|i| &rows[(i as f64 * step) as usize])
        .collect();

    info!("sampled {} domains (step={step:.1})", sampled.len());

    // Resolve in parallel
    let resolved: Vec<(&str, &str, Option<String>)> = sampled
        .par_iter()
        .map(|(rank, domain)| {
            let ip = resolve(domain);
            if ip.is_none() {
                warn!("failed to resolve {domain}");
            }
            (rank.as_str(), domain.as_str(), ip)
        })
        .collect();

    let ok = resolved.iter().filter(|(_, _, ip)| ip.is_some()).count();
    info!("resolved {ok}/{} domains", resolved.len());

    // Write output
    let out_file = File::create(&args.output).expect("failed to create output CSV");
    let mut wtr = csv::Writer::from_writer(out_file);
    wtr.write_record([
        "batch_num",
        "ip",
        "hostname",
        "rtt_ms",
        "is_http",
        "had_date",
        "clock_offset_ms",
    ])
    .unwrap();

    for (_, domain, ip) in &resolved {
        if let Some(ip) = ip {
            wtr.write_record(["0", ip, domain, "10.0", "false", "true", "0"])
                .unwrap();
        }
    }
    wtr.flush().unwrap();
    info!("wrote {ok} rows to {}", args.output.display());
}
