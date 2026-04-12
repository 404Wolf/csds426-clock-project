use clap::Parser;
use log::info;

#[derive(Parser, Debug)]
#[command(name = "http-syncing")]
#[command(about = "Estimate local clock desync using HTTP Date header", long_about = None)]
struct Args {
    /// Path to file with one host/URL per line
    #[arg(required = true)]
    input: std::path::PathBuf,

    /// Path to write results to
    #[arg(required = true)]
    output: std::path::PathBuf,
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    let hosts: Vec<String> = std::fs::read_to_string(&args.input)
        .expect("failed to read input")
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s != "host" && s != "url" && s != "ip")
        .map(|s| {
            if s.starts_with("http://") || s.starts_with("https://") {
                s
            } else {
                format!("https://{s}")
            }
        })
        .collect();

    let mut wtr = csv::Writer::from_path(&args.output).expect("failed to open output");
    wtr.write_record(["host", "clock_offset_ms"]).unwrap();

    for host in &hosts {
        info!("measuring {host}");
        match clocks::measure_host(host) {
            Ok(Some(offset)) => {
                info!("{host}: {}ms", offset.num_milliseconds());
                wtr.write_record([host.as_str(), &offset.num_milliseconds().to_string()]).unwrap();
            }
            Ok(None) => {
                info!("{host}: frozen clock");
            }
            Err(e) => {
                info!("{host}: failed — {e}");
            }
        }
        wtr.flush().unwrap();
    }
}
