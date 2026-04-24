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

test-http HOST *ARGS:
    #!/usr/bin/env bash
    extra="{{ARGS}}"
    RUSTFLAGS="-C target-cpu=native" cargo run --release --bin test-http -- {{HOST}} ${extra#-- }

tranco INPUT OUTPUT *ARGS:
    RUSTFLAGS="-C target-cpu=native" cargo run --release --bin tranco -- {{INPUT}} {{OUTPUT}} {{ARGS}}

enrich-http INPUT OUTPUT *ARGS:
    RUSTFLAGS="-C target-cpu=native" cargo run --release --bin enrich-http -- {{INPUT}} {{OUTPUT}} {{ARGS}}

analyze-all:
    #!/usr/bin/env bash
    set -e
    rm -rf analysis/out
    mkdir -p analysis/out
    cd analysis
    echo "==> icmp_vs_http"
    uv run python icmp_vs_http.py ../data/icmp_with_http.csv
    echo "==> accuracy_comparison"
    uv run python accuracy_comparison.py ../data/icmp_with_http.csv
    echo "==> search_convergence"
    uv run python search_convergence.py ../data/icmp_with_http.csv
    echo "==> tranco_rank_vs_offset"
    uv run python tranco_rank_vs_offset.py ../data/tranco_http.csv ../data/tranco_20k_sample.csv
    echo "==> clock_by_country_icmp_clockdiff_with_http"
    uv run python clock_by_country.py ../data/icmp_with_http.csv > out/clock_by_country_icmp_with_http.txt
    echo "==> clock_by_country_all_icmp_clockdiff"
    uv run python clock_by_country.py ../data/icmp_timestamp/icmp_timestamp.csv > out/clock_by_country_icmp_clockdiff.txt
    cat out/clock_by_country.txt
    echo "==> clock_skew_pct"
    uv run python clock_skew_pct.py > out/clock_skew_pct.txt
    cat out/clock_skew_pct.txt
    echo "==> http_better_than_icmp"
    uv run python http_better_than_icmp.py > out/http_better_than_icmp.csv
    echo "==> timestamp_only_examples"
    uv run python timestamp_only_examples.py > out/timestamp_only_examples.txt
    cat out/timestamp_only_examples.txt
    echo "==> icmp_overlap"
    uv run python icmp_overlap.py ../data/icmp_echo/icmp_echo.csv ../data/icmp_timestamp/icmp_timestamp.csv
    echo ""
    echo "Done. Outputs in analysis/out/"

data-slides-preview:
    cd slides && quarto preview data-report.qmd

intro-slides-preview:
    cd slides && quarto preview intro-report.qmd
