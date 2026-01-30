#!/usr/bin/env bash

set -euo pipefail

if [ "$#" -lt 1 ]; then
  echo "Usage: $0 <scan_speed> [do_test]"
  exit 1
fi

SCAN_SPEED="$1"
DO_TEST="${2:-1}"

if [ "$DO_TEST" -eq 1 ]; then
  # Fetch live IPs for Google and Amazon, since we know they are web servers
  GOOGLE_IPS=$(dig +short google.com A | head -5 | awk '{printf "%s/32%s", $0, (NR<5 && NF ? "," : "")}')
  AMAZON_IPS=$(dig +short amazon.com A | head -5 | awk '{printf "%s/32%s", $0, (NR<5 && NF ? "," : "")}')
  TEST_TARGET="$GOOGLE_IPS,$AMAZON_IPS"
  echo "Testing mode enabled. Scanning the following IPs: $TEST_TARGET"

  sudo zmap -p 80 -o "data/http.csv" "$TEST_TARGET" -r "$SCAN_SPEED" --output-module=csv
  sudo zmap --probe-module=icmp_echoscan -o "data/icmp.csv" "$TEST_TARGET" -r "$SCAN_SPEED" --output-module=csv
else
  sudo zmap -p 80 -o "data/http.csv" 0.0.0.0/0 -r "$SCAN_SPEED" --output-module=csv
  sudo zmap --probe-module=icmp_echoscan -o "data/icmp.csv" 0.0.0.0/0 -r "$SCAN_SPEED" --output-module=csv
fi

# zmap runs as root so the output files are owned by root
sudo chown "$USER" "data/http.csv" "data/icmp.csv"

# Strip headers from the CSV files
sed -i '1d' "data/http.csv"
sed -i '1d' "data/icmp.csv"

# Combine the csvs
cat "data/icmp.csv" >>"data/http.csv"
rm "data/icmp.csv"
