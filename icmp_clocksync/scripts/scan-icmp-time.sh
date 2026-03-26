#!/usr/bin/env bash

set -euo pipefail

# Source shared functions
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/shared.sh"

DO_TEST="${1:-1}"

if [ "$DO_TEST" -eq 1 ]; then
  TARGET=(-w "$(get_test_ips_file)")
else
  TARGET=(0.0.0.0/0)
fi

zmap --probe-module=icmp_timestamp "${TARGET[@]}" --output-module=csv --output-fields="*" -r 100000 | gzip >"data/icmp_timestamp.csv.gz"
chown "$USER" "data/icmp_timestamp.csv.gz"
echo "ICMP timestamp scan complete. Results saved to data/icmp_timestamp.csv.gz"

zmap --probe-module=icmp_echoscan "${TARGET[@]}" --output-module=csv --output-fields="*" -r 100000 | gzip >"data/icmp_echo.csv.gz"
chown "$USER" "data/icmp_echo.csv.gz"
echo "ICMP echo scan complete. Results saved to data/icmp_echo.csv.gz"
