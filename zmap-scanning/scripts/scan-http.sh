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
  sudo zmap -p 80 -o "data/http.csv" -I "$TEST_TARGET" -r "$SCAN_SPEED" --output-module=csv
  sudo zmap -p 443 -o "data/https.csv" -I "$TEST_TARGET" -r "$SCAN_SPEED" --output-module=csv
else
  sudo zmap -p 80 -o "data/http.csv" 0.0.0.0/0 -r "$SCAN_SPEED" --output-module=csv
  sudo zmap -p 443 -o "data/https.csv" 0.0.0.0/0 -r "$SCAN_SPEED" --output-module=csv
fi

# zmap runs as root so the output files are owned by root
sudo chown "$USER" "data/http.csv" "data/https.csv"

# Strip headers from the CSV files
sed -i '1d' "data/http.csv"
sed -i '1d' "data/https.csv"

# Combine the csvs
cat "data/https.csv" >>"data/http.csv"
rm "data/https.csv"
