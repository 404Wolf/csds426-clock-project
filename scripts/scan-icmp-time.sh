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

MODULE_NAME="icmp_timestamp"

if [ "$DO_TEST" -eq 1 ]; then
  TEST_TARGET=$(get_test_ips_file)
  sudo zmap --probe-module="$MODULE_NAME" -o "data/icmp_timestamp.csv" -I "$TEST_TARGET" -r "$SCAN_SPEED" --output-module=csv --output-fields="*"
else
  sudo zmap --probe-module="$MODULE_NAME" -o "data/icmp_timestamp.csv" 0.0.0.0/0 -r "$SCAN_SPEED" --output-module=csv --output-fields="*"
fi

# zmap runs as root so the output files are owned by root
sudo chown "$USER" "data/icmp_timestamp.csv"

echo "ICMP timestamp scan complete. Results saved to data/icmp_timestamp.csv"
