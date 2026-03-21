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

zmap --probe-module=icmp_timestamp -o "data/icmp_timestamp.csv" "${TARGET[@]}" --output-module=csv --output-fields="*"
chown "$USER" "data/icmp_timestamp.csv"
echo "ICMP timestamp scan complete. Results saved to data/icmp_timestamp.csv"

zmap --probe-module=icmp_echoscan -o "data/icmp_echo.csv" "${TARGET[@]}" --output-module=csv --output-fields="*"
chown "$USER" "data/icmp_echo.csv"
echo "ICMP echo scan complete. Results saved to data/icmp_echo.csv"
