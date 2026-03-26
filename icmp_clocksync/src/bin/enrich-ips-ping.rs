use std::fs::{File, OpenOptions};
use std::net::IpAddr;
use std::path::PathBuf;

use clap::Parser;
use icmp_clocksync::shared;
use icmp_clocksync::{get_latest_batch, iter_echo_csv, EnrichedRecord, IcmpEchoRecord};
use itertools::Itertools;
use rayon::prelude::*;

const BATCH_SIZE: usize = 100;

#[derive(Parser)]
struct Args {
    /// Input CSV file (zmap icmp_echoscan output)
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

    let geoip = maxminddb::Reader::open_readfile(&args.mmdb)
        .unwrap_or_else(|e| panic!("failed to open mmdb at {}: {e}", args.mmdb.display()));

    let resume_after = get_latest_batch(&args.output);
    let skip_rows = resume_after.map_or(0, |b| (b as usize + 1) * BATCH_SIZE);

    if let Some(b) = resume_after {
        eprintln!("resuming after batch {b}, skipping {skip_rows} rows");
    }

    let file = File::open(&args.input).expect("failed to open input csv");
    let iter = iter_echo_csv(file);

    let valid_rows = iter.filter_map(|r| match r {
        Ok(rec) if rec.success == 1 => Some(rec),
        Ok(_) => None,
        Err(e) => {
            eprintln!("skipping bad row: {e}");
            None
        }
    });

    let valid_rows: Box<dyn Iterator<Item = IcmpEchoRecord>> = if let Some(n) = args.limit {
        Box::new(valid_rows.skip(skip_rows).take(n))
    } else {
        Box::new(valid_rows.skip(skip_rows))
    };

    let append = resume_after.is_some();
    let out_file = if append {
        OpenOptions::new()
            .append(true)
            .open(&args.output)
            .expect("failed to open output csv for append")
    } else {
        File::create(&args.output).expect("failed to create output csv")
    };
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(!append)
        .from_writer(out_file);

    let start_batch = resume_after.map_or(0, |b| b + 1);
    let mut last_batch = start_batch;

    for (i, chunk) in valid_rows.chunks(BATCH_SIZE).into_iter().enumerate() {
        let batch_num = start_batch + i as u64;
        let batch: Vec<_> = chunk.collect();

        let enriched: Vec<EnrichedRecord> = batch
            .par_iter()
            .filter_map(|record| {
                let ip: IpAddr = record.saddr.parse().ok()?;
                let rtt_ms = shared::ping_rtt(ip)?;

                let hostname = shared::resolve_hostname(ip);
                let (is_http, had_date) = shared::probe_http(ip);
                let geo = shared::lookup_geo(&geoip, ip);

                Some(EnrichedRecord {
                    batch_num,
                    ip: record.saddr.clone(),
                    hostname,
                    rtt_ms,
                    is_http,
                    had_date,
                    country: geo.country,
                    city: geo.city,
                    latitude: geo.latitude,
                    longitude: geo.longitude,
                    daddr: None,
                    otime: None,
                    rtime: None,
                    ttime: None,
                    clock_offset_ms: None,
                })
            })
            .collect();

        for rec in &enriched {
            wtr.serialize(rec).expect("failed to write row");
        }
        wtr.flush().expect("failed to flush batch");
        eprintln!("batch {batch_num} done ({} rows)", enriched.len());
        last_batch = batch_num;
    }

    wtr.flush().expect("failed to flush output csv");
    eprintln!("done, last batch: {last_batch}");
}
