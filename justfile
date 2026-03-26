scan-icmp-ping NUM_IPS="200":
    sudo env PATH="$PATH" bash icmp_clocksync/scripts/scan-icmp-ping.sh {{NUM_IPS}}

scan-icmp-test:
    sudo env PATH="$PATH" bash icmp_clocksync/scripts/scan-icmp-time.sh 1

scan-icmp:
    sudo env PATH="$PATH" bash icmp_clocksync/scripts/scan-icmp-time.sh 0

enrich-ips-clockdiff-example:
    cargo run --bin enrich-ips-clockdiff -- data/icmp_timestamp_example.csv data/icmp_timestamp_analysis_example.csv

enrich-ips-clockdiff *ARGS:
    cargo run --release --bin enrich-ips-clockdiff -- data/icmp_timestamp_real.csv data/icmp_timestamp_analysis.csv {{ARGS}}

enrich-ips-ping INPUT OUTPUT *ARGS:
    cargo run --release --bin enrich-ips-ping -- {{INPUT}} {{OUTPUT}} {{ARGS}}
