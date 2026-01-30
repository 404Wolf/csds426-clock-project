use clap::Parser;
use rayon::prelude::*;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::OnceLock;
use ureq;
use ureq::Agent;
use ureq::tls::{TlsConfig, TlsProvider};

static AGENT: OnceLock<Agent> = OnceLock::new();

fn get_agent() -> &'static Agent {
    AGENT.get().expect("AGENT not initialized")
}

#[derive(Parser, Debug)]
#[command(name = "csv-loader")]
#[command(about = "Load CSV files containing IP addresses", long_about = None)]
struct Args {
    /// Path to CSV file
    #[arg(required = true)]
    data: PathBuf,

    /// Number of threads to use for parallel processing
    #[arg(short, long)]
    threads: Option<usize>,
}

fn get_dates(ip_addrs: Vec<String>) -> Vec<std::time::SystemTime> {
    ip_addrs
        .par_iter()
        .filter_map(|ip| {
            let url = format!("https://{}", ip); // Use https for TLS

            match get_agent().get(&url).call() {
                Ok(resp) => {
                    if let Some(date) = resp.headers().get("date") {
                        // Parse the HTTP date format (RFC 2822)
                        if let Ok(parsed_date) = httpdate::parse_http_date(date.to_str().unwrap()) {
                            let parsed_time = std::time::SystemTime::from(parsed_date);
                            return Some(parsed_time);
                        }
                    }
                    None
                }
                Err(e) => {
                    eprintln!("  Error making request to {}: {}", url, e);
                    None
                }
            }
        })
        .collect()
}

fn main() {
    let args = Args::parse();

    if let Some(num_threads) = args.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build_global()
            .expect("Failed to build thread pool");
    }

    AGENT.get_or_init(|| {
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
    });

    match File::open(&args.data) {
        Ok(mut file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();

            let ip_addrs: Vec<String> = contents
                .lines()
                .filter_map(|line| {
                    let ip = line.trim().to_string();
                    if !ip.is_empty() { Some(ip) } else { None }
                })
                .collect();

            let dates = get_dates(ip_addrs);

            for date in dates {
                println!(
                    "  Date: {}",
                    date.duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                );
            }
        }
        Err(e) => eprintln!("Error opening file {}: {}", args.data.display(), e),
    }
}
