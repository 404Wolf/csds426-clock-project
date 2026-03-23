use std::fs::{File, OpenOptions};
use std::net::IpAddr;
use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use dns_lookup::lookup_addr;
use icmp_clocksync::{get_latest_batch, iter_icmp_csv, IcmpTimestampRecord, ResolvedRecord};
use rayon::prelude::*;

const BATCH_SIZE: usize = 100;

#[derive(Parser)]
struct Args {
    /// Input CSV file
    input: PathBuf,
    /// Output CSV file
    output: PathBuf,
}

fn main() {
    let args = Args::parse();
    let input_path = args.input;
    let output_path = args.output;

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
        OpenOptions::new().append(true).open(&output_path).expect("failed to open output csv for append")
    } else {
        File::create(&output_path).expect("failed to create output csv")
    };
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(!append)
        .from_writer(out_file);

    let mut batch: Vec<IcmpTimestampRecord> = Vec::with_capacity(BATCH_SIZE);
    let start_batch = resume_after.map_or(0, |b| b + 1);
    let mut batch_num = start_batch;

    let valid_rows = iter.filter_map(|r| match r {
        Ok(rec) => Some(rec),
        Err(e) => {
            eprintln!("skipping bad row: {e}");
            None
        }
    });

    for record in valid_rows.skip(skip_rows) {
        batch.push(record);

        if batch.len() == BATCH_SIZE {
            flush_batch(&mut wtr, &batch, batch_num);
            batch.clear();
            batch_num += 1;
        }
    }

    // Flush remaining partial batch
    if !batch.is_empty() {
        flush_batch(&mut wtr, &batch, batch_num);
    }

    wtr.flush().expect("failed to flush output csv");
    eprintln!("done, last batch: {batch_num}");
}

fn flush_batch(wtr: &mut csv::Writer<File>, batch: &[IcmpTimestampRecord], batch_num: u64) {
    let resolved: Vec<ResolvedRecord> = batch
        .par_iter()
        .map(|record| {
            let hostname = record
                .saddr
                .parse::<IpAddr>()
                .ok()
                .and_then(|ip| lookup_addr(&ip).ok())
                .unwrap_or_default();
            let (is_http, had_date) = record
                .saddr
                .parse::<IpAddr>()
                .ok()
                .map(|ip| probe_http(ip))
                .unwrap_or((false, false));

            let t1 = record.otime as i64;
            let t2 = record.rtime as i64;
            let t3 = record.ttime as i64;
            let t4 = t1 + record.rtt_ms as i64;
            let clock_offset_ms = ((t2 - t1) + (t3 - t4)) / 2;

            ResolvedRecord {
                batch_num,
                saddr: record.saddr.clone(),
                hostname,
                daddr: record.daddr.clone(),
                otime: record.otime,
                rtime: record.rtime,
                ttime: record.ttime,
                rtt_ms: record.rtt_ms,
                clock_offset_ms,
                is_http,
                had_date,
            }
        })
        .collect();

    for rec in &resolved {
        wtr.serialize(rec).expect("failed to write row");
    }
    wtr.flush().expect("failed to flush batch");

    eprintln!("batch {batch_num} done ({} rows)", resolved.len());
}

/// Send a HEAD request to port 80 and return (is_http, had_date).
fn probe_http(ip: IpAddr) -> (bool, bool) {
    let agent = ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_millis(500)))
        .timeout_response(Some(Duration::from_millis(500)))
        .build()
        .new_agent();

    match agent.head(&format!("http://{ip}/")).call() {
        Ok(resp) => {
            let had_date = resp.headers().get("date").is_some();
            (true, had_date)
        }
        Err(ureq::Error::StatusCode(_)) => {
            // Got an HTTP error response — server is up, check for Date header
            // ureq doesn't expose headers on error responses in v3, so conservatively false
            (true, false)
        }
        Err(_) => (false, false),
    }
}
