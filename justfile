scan-icmp-test:
    sudo bash icmp_clocksync/scripts/scan-icmp-time.sh 1

scan-icmp:
    sudo bash icmp_clocksync/scripts/scan-icmp-time.sh 0

analyze-icmp-example:
    cargo run --bin analyze-icmp -- data/icmp_timestamp_example.csv data/icmp_timestamp_analysis_example.csv

analyze-icmp:
    cargo run --release --bin analyze-icmp -- data/icmp_timestamp_real.csv data/icmp_timestamp_analysis.csv
