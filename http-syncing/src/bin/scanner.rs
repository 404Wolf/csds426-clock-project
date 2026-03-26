use chrono::DateTime;
use clap::Parser;
use rayon::prelude::*;
use ureq::Agent;
use ureq::tls::{TlsConfig, TlsProvider};

#[derive(Parser, Debug)]
#[command(name = "http-syncing")]
#[command(about = "Estimate local clock desync using HTTP Date header", long_about = None)]
struct Args {
    /// Path to CSV file with one IPv4/domain per row
    #[arg(required = true)]
    input: std::path::PathBuf,

    /// Path to write CSV output to
    #[arg(required = true)]
    output: std::path::PathBuf,

    /// Number of times to scan each host
    #[arg(short, long, default_value_t = 1)]
    runs: u32,
}

fn make_agent() -> Agent {
    Agent::new_with_config(
        ureq::config::Config::builder()
            .tls_config(
                TlsConfig::builder()
                    .provider(TlsProvider::Rustls)
                    .disable_verification(true)
                    .build(),
            )
            .http_status_as_error(false)
            .redirect_auth_headers(ureq::config::RedirectAuthHeaders::SameHost)
            .build(),
    )
}

fn main() {
    let args = Args::parse();

    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .from_path(&args.input)
        .expect("failed to open input file");

    let hosts: Vec<String> = rdr
        .records()
        .filter_map(|r| r.ok())
        .filter_map(|r| r.get(0).map(|s| s.trim().to_string()))
        .filter(|s| !s.is_empty() && s != "host" && s != "url" && s != "ip")
        .map(|s| {
            if s.starts_with("http://") || s.starts_with("https://") {
                s
            } else {
                format!("https://{}", s)
            }
        })
        .collect();

    let work: Vec<(String, u32)> = hosts
        .iter()
        .flat_map(|url| (0..args.runs).map(move |n| (url.clone(), n)))
        .collect();

    let mut results: Vec<(
        String,
        u32,
        i64,
        DateTime<chrono::Utc>,
        DateTime<chrono::Utc>,
        DateTime<chrono::Utc>,
    )> = work
        .par_iter()
        .flat_map(|(url, run_num)| {
            let agent = make_agent();
            let rtt_estimate = estimate_rtt(&agent, url);

            let step_micros: i64 = 300; // ~0.3ms
            let entries: i64 = 100;

            let rtt_i64 = rtt_estimate as i64;
            let center = -rtt_i64;

            // Choose a random window of 100 entries around the center (-rtt estimate)
            let half_span = (entries / 2) * step_micros;
            let jitter = rand::random::<i64>().abs() % step_micros.max(1);
            let start = (center - half_span) + jitter;

            let hit_at = (0..entries)
                .map(|n| start + n * step_micros)
                .collect::<Vec<_>>();
            let rows: Vec<(
                String,
                u32,
                i64,
                DateTime<chrono::Utc>,
                DateTime<chrono::Utc>,
                DateTime<chrono::Utc>,
            )> = hit_at
                .into_par_iter()
                .map(|i| {
                    let req_url = format!("{}?q={}", url, rand::random::<u64>());
                    let (server, sent_at, receive_at) =
                        sleep_to_edge_and_get_date(&agent, req_url.as_str(), i);
                    (url.clone(), *run_num, i, server, sent_at, receive_at)
                })
                .collect();
            rows
        })
        .collect();

    results.sort_by_key(|(host, run_num, _, server, sent_at, receive_at)| {
        (host.clone(), *run_num, *server, *sent_at, *receive_at)
    });

    let mut wtr = csv::Writer::from_path(&args.output).expect("failed to open output file");
    for (host, run_num, i, server, sent_at, receive_at) in &results {
        wtr.serialize(clocks::Record {
            host: host.clone(),
            run_num: *run_num,
            offset_micros: *i,
            server: *server,
            send_at: *sent_at,
            receive_at: *receive_at,
        })
        .expect("failed to write record");
    }
    wtr.flush().expect("failed to flush");
}

/// Sleeps until the end of the current second plus some offset, and then
/// requests the server's date. Returns the server's date, send_at, and receive_at.
fn sleep_to_edge_and_get_date(
    agent: &Agent,
    url: &str,
    offset_micros: i64,
) -> (
    DateTime<chrono::Utc>,
    DateTime<chrono::Utc>,
    DateTime<chrono::Utc>,
) {
    let time_now = chrono::Utc::now();

    // Sleep until the next second boundary (+ offset)
    let micros_until_next_second =
        1_000_000i64 - (time_now.timestamp_subsec_micros() as i64 % 1_000_000i64);
    let total_micros = (micros_until_next_second + offset_micros).max(0);

    spin_sleep::sleep(std::time::Duration::from_micros(total_micros as u64));
    let sent_at = chrono::Utc::now();

    let resp = agent.head(url).call().expect("failed to make request");
    let receive_at = chrono::Utc::now();

    let date_header = resp
        .headers()
        .get("date")
        .expect("didn't have date header")
        .to_str()
        .expect("failed to convert date header to string");

    let reported_date =
        httpdate::parse_http_date(date_header).expect("failed to parse date header");
    let reported_date: DateTime<chrono::Utc> = DateTime::<chrono::Utc>::from(reported_date);

    (reported_date, sent_at, receive_at)
}

/// Estimate the RTT to the host by making 5 requests and taking the average.
/// Returns the average RTT in microseconds.
fn estimate_rtt(agent: &Agent, url: &str) -> u128 {
    let mut rtt_sum_micros: u128 = 0;

    for _ in 0..5 {
        let start = std::time::Instant::now();
        agent.head(url).call().expect("failed to make request");
        rtt_sum_micros += start.elapsed().as_micros();
    }

    rtt_sum_micros / 5
}
