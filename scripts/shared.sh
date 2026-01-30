#!/usr/bin/env bash

# Get a list of test IPs from known reliable hosts
# Returns: Comma-separated list of IPs in CIDR notation (e.g., "1.2.3.4/32,5.6.7.8/32")
get_test_ips() {
  local google_ips case_ips amazon_ips

  google_ips=$(dig +short google.com A | head -1 | awk '{printf "%s/32%s", $0, (NR<5 && NF ? "," : "")}')
  case_ips=$(dig +short case.edu A | head -5 | awk '{printf "%s/32%s", $0, (NR<5 && NF ? "," : "")}')
  amazon_ips=$(dig +short amazon.com A | head -5 | awk '{printf "%s/32%s", $0, (NR<5 && NF ? "," : "")}')

  local all_ips="${google_ips},${case_ips},${amazon_ips}"
  echo "${all_ips}" | sed 's/,\+/,/g' | sed 's/^,//;s/,$//'
}
