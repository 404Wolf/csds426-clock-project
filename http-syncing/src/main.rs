use chrono::DateTime;
use clap::Parser;
use ureq::Agent;
use ureq::tls::{TlsConfig, TlsProvider};

#[derive(Parser, Debug)]
#[command(name = "http-syncing")]
#[command(about = "Estimate local clock desync using HTTP Date header", long_about = None)]
struct Args {
    /// URL to request (e.g. https://example.com or example.com)
    #[arg(required = true)]
    url: String,
}

fn main() {
    let args = Args::parse();

    let url = if args.url.starts_with("http://") || args.url.starts_with("https://") {
        args.url
    } else {
        format!("https://{}", args.url)
    };

    // Keep behavior similar to prior code that allowed invalid certs.
    // Consider removing this for production use.
    let agent = Agent::new_with_config(
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
    );
    let mut results: Vec<(i64, DateTime<chrono::Utc>, DateTime<chrono::Utc>)> =
        Vec::with_capacity(50);

    let rtt_estimate = estimate_rtt(&agent, &url);

    std::thread::scope(|s| {
        let mut handles = Vec::with_capacity(50);
        for i in -(((rtt_estimate / 2) as i64) + 500)..=100 {
            if i % 1000 != 0 {
                continue;
            }

            let i = i * 10;

            let agent = &agent;
            let url: &str = &url;

            handles.push(s.spawn(move || {
                let (server, sent_at) = sleep_to_edge_and_get_date(agent, url, i);
                (i, server, sent_at)
            }));
        }

        for h in handles {
            results.push(h.join().expect("scoped thread panicked"));
        }
    });

    results.sort_by_key(|(_, server, sent_at)| (*server, *sent_at));

    println!("offset_micros,server,sent_at");
    for (i, server, sent_at) in &results {
        println!("{},{},{}", i, server.to_rfc3339(), sent_at.to_rfc3339());
    }
}

/// Sleeps until the end of the current second plus some offset, and then
/// requests the server's date. Returns the server's date, and the sent at date.
fn sleep_to_edge_and_get_date(
    agent: &Agent,
    url: &str,
    offset_micros: i64,
) -> (DateTime<chrono::Utc>, DateTime<chrono::Utc>) {
    let time_now = chrono::Utc::now();

    // Sleep until the next second boundary (+ offset)
    let micros_until_next_second =
        1_000_000i64 - (time_now.timestamp_subsec_micros() as i64 % 1_000_000i64);
    let total_micros = (micros_until_next_second + offset_micros).max(0);

    spin_sleep::sleep(std::time::Duration::from_micros(total_micros as u64));
    let sent_at = chrono::Utc::now();

    let resp = agent.head(url).call().expect("failed to make request");

    let date_header = resp
        .headers()
        .get("date")
        .expect("didn't have date header")
        .to_str()
        .expect("failed to convert date header to string");

    let reported_date =
        httpdate::parse_http_date(date_header).expect("failed to parse date header");
    let reported_date: DateTime<chrono::Utc> = DateTime::<chrono::Utc>::from(reported_date);

    (reported_date, sent_at)
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
