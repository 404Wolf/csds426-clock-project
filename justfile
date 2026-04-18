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

data-slides-preview:
    cd slides && quarto preview data-report.qmd

intro-slides-preview:
    cd slides && quarto preview intro-report.qmd
