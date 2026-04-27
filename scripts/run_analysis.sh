#!/usr/bin/env bash
set -e
cd "$(dirname "$0")/../analysis"

# icmp_with_http analysis
uv run python icmp_vs_http.py ../data/icmp_with_http.csv
uv run python accuracy_comparison.py ../data/icmp_with_http.csv
uv run python search_convergence.py ../data/icmp_with_http.csv
uv run python clock_by_country.py ../data/icmp_with_http.csv
uv run python clock_skew_pct.py
uv run python http_better_than_icmp.py
uv run python geo_plot.py ../data/icmp_with_http.csv ../data/icmp_timestamp_enriched_clean.csv

# tranco / ICMP-only analysis
uv run python tranco_rank_vs_offset.py ../data/tranco_http.csv ../data/tranco_20k_sample.csv
uv run python icmp_overlap.py ../data/icmp_echo/icmp_echo.csv ../data/icmp_timestamp/icmp_timestamp.csv
