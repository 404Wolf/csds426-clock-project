scan-icmp-ping NUM_IPS="200":
    sudo env PATH="$PATH" bash icmp_clocksync/scripts/scan-icmp-ping.sh {{NUM_IPS}}

scan-icmp-test:
    sudo env PATH="$PATH" bash icmp_clocksync/scripts/scan-icmp-time.sh 1

scan-icmp:
    sudo env PATH="$PATH" bash icmp_clocksync/scripts/scan-icmp-time.sh 0

enrich-ips-clockdiff-example INPUT="data/icmp_timestamp_example.csv" OUTPUT="data/icmp_timestamp_analysis_example.csv":
    cargo run --bin enrich-ips-clockdiff -- {{INPUT}} {{OUTPUT}}

enrich-ips-clockdiff INPUT OUTPUT *ARGS:
    cargo run --release --bin enrich-ips-clockdiff -- {{INPUT}} {{OUTPUT}} {{ARGS}}

enrich-ips-ping INPUT OUTPUT *ARGS:
    cargo run --release --bin enrich-ips-ping -- {{INPUT}} {{OUTPUT}} {{ARGS}}

scan-http INPUT OUTPUT *ARGS:
    cd http-syncing && cargo run --release --bin scanner -- {{INPUT}} {{OUTPUT}} {{ARGS}}
