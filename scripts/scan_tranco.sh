#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$SCRIPT_DIR/.."
DATA="$ROOT/data"

WHITELIST=$(mktemp)
trap 'rm -f "$WHITELIST"' EXIT

# Extract IPs from tranco_http.csv (skip header, col 2)
tail -n +2 "$DATA/tranco_http.csv" | cut -d, -f2 > "$WHITELIST"

TIMESTAMP=$(date +%Y%m%d_%H%M%S)

zmap --probe-module=icmp_echoscan -w "$WHITELIST" --output-module=csv --output-fields="*" -r 100000 \
    > "$DATA/tranco_icmp_echo_$TIMESTAMP.csv"

zmap --probe-module=icmp_timestamp -w "$WHITELIST" --output-module=csv --output-fields="*" -r 100000 \
    > "$DATA/tranco_icmp_timestamp_$TIMESTAMP.csv"
