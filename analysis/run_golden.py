#!/usr/bin/env python3
"""Continuously run test-http with golden params and log results to CSV."""

import argparse
import csv
import os
import re
import subprocess
from itertools import cycle
from pathlib import Path

OFFSET_RE = re.compile(r"http_clock_offset_us=(-?\d+)us")

GOLDEN_FLAGS = [
    "--rounds", "17",
    "--probes", "18",
    "--initial-half-span-us", "1750000",
    "--min-step-us", "4300",
    "--shrink-factor", "5",
    "--method", "HEAD",
    "--best-of", "2",
]

OFFSETS_S = [-1, -0.1, -0.01, 0, 0.01, 0.1, 1]

CSV_FILE = Path(__file__).parent.parent / "data" / "golden_measurements.csv"
CSV_FIELDS = ["run", "host", "offset_s", "expected_us", "measured_us", "err_us"]


def ensure_csv():
    if not os.path.exists(CSV_FILE):
        with open(CSV_FILE, "w", newline="") as f:
            csv.writer(f).writerow(CSV_FIELDS)


def next_run():
    if not os.path.exists(CSV_FILE):
        return 0
    with open(CSV_FILE) as f:
        rows = list(csv.DictReader(f))
    return max((int(r["run"]) for r in rows), default=-1) + 1


def measure(host: str, offset_s: float, timeout: int = 120) -> int | None:
    url = f"http://{host}/{offset_s}"
    cmd = ["just", "test-http", url, "--"] + GOLDEN_FLAGS
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
        m = OFFSET_RE.search(r.stdout + r.stderr)
        return int(m.group(1)) if m else None
    except subprocess.TimeoutExpired:
        return None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("hosts", nargs="+", help="host:port of fake time servers")
    args = ap.parse_args()

    ensure_csv()
    run = next_run()

    print(f"Starting from run {run}. Writing to {CSV_FILE}. Ctrl-C to stop.")

    jobs = cycle([(host, off) for off in OFFSETS_S for host in args.hosts])

    for host, offset_s in jobs:
        expected_us = int(offset_s * 1_000_000)
        result_us = measure(host, offset_s)
        if result_us is None:
            print(f"run {run} {host} offset={offset_s:+.3f}s -> FAIL")
        else:
            err = abs(result_us - expected_us)
            print(f"run {run} {host} offset={offset_s:+.3f}s -> measured={result_us:+d}us expected={expected_us:+d}us err={err}us")
            with open(CSV_FILE, "a", newline="") as f:
                csv.writer(f).writerow([run, host, offset_s, expected_us, result_us, err])
        run += 1


if __name__ == "__main__":
    main()
