use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;

use clap::Parser;
use log::{info, warn};
use rayon::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const BATCH_SIZE: usize = 4;

#[derive(Serialize)]
struct ProbeRow<'a> {
    host: &'a str,
    round: u32,
    request: u32,
    offset_micros: i64,
    send_at: DateTime<Utc>,
    receive_at: DateTime<Utc>,
    server: DateTime<Utc>,
    rtt_us: i64,
}

#[derive(Parser)]
#[command(about = "Measure HTTP clock offset for hosts with ICMP timestamp data")]
struct Args {
    /// Enriched CSV from enrich-ips-clockdiff
    input: PathBuf,
    /// Output comparison CSV
    output: PathBuf,
    /// HTTP method to use for probing (e.g. HEAD, GET)
    #[arg(long, default_value = "HEAD")]
    method: String,
    /// Number of binary-search rounds per host
    #[arg(long, default_value_t = 10)]
    rounds: u32,
    /// Number of probes fired per round
    #[arg(long, default_value_t = 10)]
    probes: i64,
    /// Initial half-span of the search window in microseconds
    #[arg(long, default_value_t = 1_300_000)]
    initial_half_span_us: i64,
    /// Stop recursing when the probe step drops below this many microseconds
    #[arg(long, default_value_t = 1_000)]
    min_step_us: i64,
    /// Skip binary search and report raw offset if server clock differs by more than this many seconds
    #[arg(long, default_value_t = 5)]
    sanity_max_offset_secs: i64,
    /// If set, write every individual probe to this CSV for debugging
    #[arg(long)]
    probe_csv: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct InputRecord {
    #[serde(default)]
    batch_num: u64,
    /// IP to probe — "ip" in enriched format, "saddr" in raw ICMP scan format
    #[serde(alias = "saddr")]
    ip: String,
    #[serde(default)]
    hostname: String,
    #[serde(default)]
    rtt_ms: f64,
    #[serde(default)]
    is_http: bool,
    /// Enriched format: host had a valid HTTP Date header during ICMP scan
    #[serde(default)]
    had_date: bool,
    /// Raw ICMP format: host replied to timestamp request (1=yes, 0=no)
    #[serde(default)]
    success: u8,
    #[serde(default)]
    country: String,
    #[serde(default)]
    city: String,
    #[serde(default)]
    latitude: f64,
    #[serde(default)]
    longitude: f64,
    #[serde(default)]
    clock_offset_ms: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct BatchOnly {
    batch_num: u64,
}

fn get_latest_batch(path: &PathBuf) -> Option<u64> {
    if !path.is_file() {
        return None;
    }
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

fn build_header(num_rounds: u32) -> Vec<String> {
    let mut h: Vec<String> = vec![
        "batch_num", "ip", "hostname", "icmp_rtt_ms", "icmp_clock_offset_ms",
        "http_clock_offset_ms", "is_frozen_clock", "country", "city", "latitude", "longitude",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();

    for r in 1..=num_rounds {
        h.push(format!("round_{r}_diff_ms"));
        h.push(format!("round_{r}_window_ms"));
        h.push(format!("round_{r}_center_us"));
    }
    h
}

fn main() {
    env_logger::init();

    rayon::ThreadPoolBuilder::new()
        .num_threads(20)
        .build_global()
        .unwrap();

    let args = Args::parse();

    let cfg = clocks::SearchConfig {
        num_rounds: args.rounds,
        probes: args.probes,
        initial_half_span_us: args.initial_half_span_us,
        min_step_us: args.min_step_us,
        sanity_max_offset_secs: args.sanity_max_offset_secs,
    };

    let resume_after = get_latest_batch(&args.output);
    let skip_rows = resume_after.map_or(0, |b| (b as usize + 1) * BATCH_SIZE);

    if let Some(b) = resume_after {
        info!("resuming after batch {b}, skipping {skip_rows} rows");
    }

    let in_file = File::open(&args.input).expect("failed to open input CSV");
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(std::io::BufReader::new(in_file));

    let all_candidates: Vec<InputRecord> = rdr
        .deserialize::<InputRecord>()
        .filter_map(|r| match r {
            Ok(rec) => Some(rec),
            Err(e) => { warn!("skipping malformed row: {e}"); None }
        })
        .filter(|r| (r.had_date && r.clock_offset_ms.is_some()) || r.success == 1)
        .collect();

    info!("{} candidates pass filter", all_candidates.len());

    let candidates: Vec<InputRecord> = all_candidates.into_iter().skip(skip_rows).collect();

    let append = resume_after.is_some();
    let out_file = if append {
        OpenOptions::new()
            .append(true)
            .open(&args.output)
            .expect("failed to open output CSV for append")
    } else {
        File::create(&args.output).expect("failed to create output CSV")
    };

    let mut wtr = csv::Writer::from_writer(out_file);

    if !append {
        wtr.write_record(&build_header(cfg.num_rounds)).expect("failed to write header");
    }

    let mut probe_wtr = args.probe_csv.as_ref().map(|path| {
        csv::Writer::from_path(path).expect("failed to open probe CSV")
    });

    let start_batch = resume_after.map_or(0, |b| b + 1);

    for (i, chunk) in candidates.chunks(BATCH_SIZE).enumerate() {
        let batch_num = start_batch + i as u64;

        info!("batch {batch_num} ({} hosts)", chunk.len());

        let results: Vec<(Option<Vec<String>>, Vec<clocks::Record>)> = chunk
            .par_iter()
            .map(|rec| {
                let url = format!("http://{}", rec.ip);
                info!("measuring {}", rec.ip);
                match clocks::measure_host_with_config(&url, &args.method, &cfg) {
                    Ok(result) => {
                        let frozen = result.offset.is_none();
                        info!(
                            "{} icmp_offset={}ms http_offset={}",
                            rec.ip,
                            rec.clock_offset_ms.unwrap_or(0),
                            result.offset.map_or("frozen".to_string(), |d| format!("{}ms", d.num_milliseconds())),
                        );

                        // Index rounds by round_num for O(1) lookup
                        let round_map: HashMap<u32, &clocks::RoundResult> =
                            result.rounds.iter().map(|r| (r.round_num, r)).collect();

                        let mut row: Vec<String> = vec![
                            batch_num.to_string(),
                            rec.ip.clone(),
                            rec.hostname.clone(),
                            rec.rtt_ms.to_string(),
                            rec.clock_offset_ms.unwrap_or(0).to_string(),
                            result.offset.map_or(String::new(), |d| d.num_milliseconds().to_string()),
                            frozen.to_string(),
                            rec.country.clone(),
                            rec.city.clone(),
                            rec.latitude.to_string(),
                            rec.longitude.to_string(),
                        ];

                        for n in 1..=cfg.num_rounds {
                            if let Some(r) = round_map.get(&n) {
                                row.push(r.diff_ms.map_or(String::new(), |d| d.to_string()));
                                row.push(r.window_half_ms.to_string());
                                row.push(r.center_us.to_string());
                            } else {
                                row.push(String::new());
                                row.push(String::new());
                                row.push(String::new());
                            }
                        }

                        (Some(row), result.probes)
                    }
                    Err(e) => {
                        warn!("{} failed, skipping: {e}", rec.ip);
                        (None, vec![])
                    }
                }
            })
            .collect();

        let succeeded = results.iter().filter(|(r, _)| r.is_some()).count();
        for (row, probes) in results {
            if let Some(row) = row {
                wtr.write_record(&row).expect("failed to write row");
            }
            if let Some(ref mut pw) = probe_wtr {
                for p in &probes {
                    pw.serialize(ProbeRow {
                        host: &p.host,
                        round: p.run_num,
                        request: p.request_num,
                        offset_micros: p.offset_micros,
                        send_at: p.send_at,
                        receive_at: p.receive_at,
                        server: p.server,
                        rtt_us: (p.receive_at - p.send_at).num_microseconds().unwrap_or(0),
                    }).expect("failed to write probe row");
                }
                pw.flush().expect("failed to flush probe CSV");
            }
        }
        wtr.flush().expect("failed to flush");
        info!("batch {batch_num} done ({succeeded}/{} succeeded)", chunk.len());
    }
}
