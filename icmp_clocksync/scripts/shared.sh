#!/usr/bin/env bash

# Get a file that has a list of test IPs from known reliable hosts
get_test_ips_file() {
  local temp_file
  temp_file=$(mktemp /tmp/test_ips.XXXXXX)

  dig +short google.com A | head -1 >> "${temp_file}"
  dig +short case.edu A | head -5 >> "${temp_file}"
  dig +short amazon.com A | head -5 >> "${temp_file}"

  echo "${temp_file}"
}
