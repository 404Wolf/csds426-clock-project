#!/usr/bin/env bash

set -euo pipefail

# Source shared functions
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/shared.sh"

if [ "$#" -lt 1 ]; then
  echo "Usage: $0 <scan_speed> [do_test]"
  exit 1
fi

SCAN_SPEED="$1"
DO_TEST="${2:-1}"

if [ "$DO_TEST" -eq 1 ]; then
  TEST_TARGET=$(get_test_ips_file)
  sudo zmap -p 80 -o "data/http.csv" "$TEST_TARGET" -r "$SCAN_SPEED" --output-module=csv
  sudo zmap --probe-module=icmp_echoscan -o "data/icmp.csv" -I "$TEST_TARGET" -r "$SCAN_SPEED" --output-module=csv --output-fields="*"
else
  sudo zmap -p 80 -o "data/http.csv" 0.0.0.0/0 -r "$SCAN_SPEED" --output-module=csv
  sudo zmap --probe-module=icmp_echoscan -o "data/icmp.csv" 0.0.0.0/0 -r "$SCAN_SPEED" --output-module=csv --output-fields="*"
fi

# zmap runs as root so the output files are owned by root
sudo chown "$USER" "data/http.csv" "data/icmp.csv"

# Strip headers from the CSV files
sed -i '1d' "data/http.csv"
sed -i '1d' "data/icmp.csv"

# Combine the csvs
cat "data/icmp.csv" >>"data/http.csv"
rm "data/icmp.csv"
