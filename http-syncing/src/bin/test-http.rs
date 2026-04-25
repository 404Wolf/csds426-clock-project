use std::path::PathBuf;

use clap::Parser;
use serde::Serialize;

#[derive(Parser)]
#[command(about = "Test HTTP clock measurement against a single host")]
struct Args {
    /// Host or URL to probe
    host: String,
    /// HTTP method to use
    #[arg(long, default_value = "HEAD")]
    method: String,
    /// Number of binary-search rounds
    #[arg(long, default_value_t = 10)]
    rounds: u32,
    /// Number of probes per round
    #[arg(long, default_value_t = 10)]
    probes: i64,
    /// Initial half-span of the search window in microseconds
    #[arg(long, default_value_t = 1_300_000)]
    initial_half_span_us: i64,
    /// Stop recursing when probe step drops below this many microseconds
    #[arg(long, default_value_t = 1_000)]
    min_step_us: i64,
    /// Skip binary search and report raw offset if server clock differs by more than this many seconds
    #[arg(long, default_value_t = 5)]
    sanity_max_offset_secs: i64,
    /// Factor by which to shrink the search window each round
    #[arg(long, default_value_t = 2)]
    shrink_factor: i64,
    /// Run the full measurement this many times and keep the one with the smallest absolute clock offset
    #[arg(long, default_value_t = 1)]
    best_of: u32,
    /// Write per-probe details to this CSV
    #[arg(long)]
    probe_csv: Option<PathBuf>,
}

#[derive(Serialize)]
struct ProbeRow {
    host: String,
    round: u32,
    request: u32,
    offset_micros: i64,
    /// Local time we sent the request (microsecond precision)
    send_at_us: i64,
    /// Local time we received the response (microsecond precision)
    receive_at_us: i64,
    /// RTT = receive_at_us - send_at_us
    rtt_us: i64,
    /// Raw second from HTTP Date header (integer Unix seconds, second resolution)
    server_unix_s: i64,
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    let url = if args.host.starts_with("http://") || args.host.starts_with("https://") {
        args.host.clone()
    } else {
        format!("http://{}", args.host)
    };

    let cfg = clocks::SearchConfig {
        num_rounds: args.rounds,
        probes: args.probes,
        initial_half_span_us: args.initial_half_span_us,
        min_step_us: args.min_step_us,
        sanity_max_offset_secs: args.sanity_max_offset_secs,
        shrink_factor: args.shrink_factor,
        best_of: 1,
    };

    let result = (0..args.best_of.max(1))
        .map(|i| {
            if args.best_of > 1 {
                eprintln!("run {}/{}", i + 1, args.best_of);
            }
            clocks::measure_host_with_config(&url, &args.method, &cfg)
        })
        .filter_map(|r| r.ok())
        .filter_map(|r| r.offset.map(|off| (off.num_microseconds().unwrap_or(i64::MAX).abs(), r)))
        .min_by_key(|(abs_us, _)| *abs_us)
        .map(|(_, r)| r);

    match result {
        None => {
            eprintln!("all runs failed or returned no offset");
            std::process::exit(1);
        }
        Some(result) => {
            match result.offset {
                Some(offset) => println!(
                    "http_clock_offset_us={}us",
                    offset.num_microseconds().unwrap_or(0)
                ),
                None => println!("frozen clock"),
            }

            if let Some(path) = &args.probe_csv {
                let mut wtr = csv::Writer::from_path(path).expect("failed to open probe CSV");
                for p in &result.probes {
                    let send_at_us = p.send_at.timestamp_micros();
                    let receive_at_us = p.receive_at.timestamp_micros();
                    wtr.serialize(ProbeRow {
                        host: p.host.clone(),
                        round: p.run_num,
                        request: p.request_num,
                        offset_micros: p.offset_micros,
                        send_at_us,
                        receive_at_us,
                        rtt_us: receive_at_us - send_at_us,
                        server_unix_s: p.server.timestamp(),
                    })
                    .expect("failed to write probe row");
                }
                wtr.flush().expect("failed to flush probe CSV");
            }
        }
    }
}
