use clap::Parser;
use csv::ReaderBuilder;
use std::fs::File;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "csv-loader")]
#[command(about = "Load CSV files containing IP addresses", long_about = None)]
struct Args {
    /// Path to CSV file
    #[arg(required = true)]
    data: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("Loading: {}", args.data.display());

    match File::open(&args.data) {
        Ok(file) => {
            let mut reader = ReaderBuilder::new().has_headers(false).from_reader(file);

            for result in reader.records() {
                match result {
                    Ok(record) => {
                        if let Some(saddr) = record.get(0) {
                            println!("{}", saddr);
                        }
                    }
                    Err(e) => eprintln!("Error reading record: {}", e),
                }
            }
        }
        Err(e) => eprintln!("Error opening file {}: {}", args.data.display(), e),
    }
}
