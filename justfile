analyze-icmp-example:
    cargo run --bin analyze-icmp -- data/icmp_timestamp_example.csv data/icmp_timestamp_analysis_example.csv

analyze-icmp:
    cargo run --release --bin analyze-icmp -- data/icmp_timestamp_real.csv data/icmp_timestamp_analysis.csv
