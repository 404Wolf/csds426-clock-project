#!/usr/bin/env bash
set -e
cd "$(dirname "$0")/../analysis"

uv run python icmp_vs_http.py ../data/icmp_with_http.csv
uv run python accuracy_comparison.py ../data/icmp_with_http.csv
uv run python search_convergence.py ../data/icmp_with_http.csv
uv run python tranco_rank_vs_offset.py ../data/tranco_http.csv ../data/tranco_20k_sample.csv
uv run python icmp_overlap.py ../data/icmp_echo/icmp_echo.csv ../data/icmp_timestamp/icmp_timestamp.csv
