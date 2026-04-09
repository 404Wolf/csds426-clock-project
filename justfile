scan-icmp-ping:
    sudo env PATH="$PATH" bash icmp_clocksync/scripts/scan-icmp-ping.sh

scan-icmp-clockdiff:
    sudo env PATH="$PATH" bash icmp_clocksync/scripts/scan-icmp-time.sh

enrich-ips-clockdiff INPUT OUTPUT *ARGS:
    cargo run --release --bin enrich-ips-clockdiff -- {{INPUT}} {{OUTPUT}} {{ARGS}}

enrich-ips-ping INPUT OUTPUT *ARGS:
    cargo run --release --bin enrich-ips-ping -- {{INPUT}} {{OUTPUT}} {{ARGS}}

scan-http INPUT OUTPUT *ARGS:
    cd http-syncing && cargo run --release --bin scanner -- {{INPUT}} {{OUTPUT}} {{ARGS}}

enrich-http INPUT OUTPUT *ARGS:
    #!/usr/bin/env bash
    tmp=$(mktemp --suffix=.csv)
    cargo run --release --bin enrich-http -- {{INPUT}} "$tmp" {{ARGS}}
    tar czf {{OUTPUT}} -C "$(dirname "$tmp")" "$(basename "$tmp")"
    rm "$tmp"

analysis-setup:
    cd analysis && uv sync

geo-plot INPUT="data/icmp_clockdiff_analysis.csv" OUTPUT="data/clock_sync_map.html" *ARGS:
    cd analysis && uv run python geo_plot.py ../{{INPUT}} -o ../{{OUTPUT}} {{ARGS}}

icmp-vs-http INPUT="output.csv" OUTPUT="data/icmp_vs_http.html" *ARGS:
    cd analysis && uv run python icmp_vs_http.py ../{{INPUT}} -o ../{{OUTPUT}} {{ARGS}}

data-slides-preview:
    cd slides && quarto preview data-report.qmd

intro-slides-preview:
    cd slides && quarto preview intro-report.qmd
