#!/usr/bin/env python3
"""Print % of hosts with clocks >5 seconds off from ICMP and HTTP scans."""

import pandas as pd
from pathlib import Path

ROOT = Path(__file__).parent.parent
THRESHOLD_MS = 5_000  # 5 seconds

# ── ICMP comprehensive scan ───────────────────────────────────────────────────
icmp_file = ROOT / "data/icmp_timestamp/icmp_timestamp.csv"

total_icmp = 0
over_icmp = 0

for chunk in pd.read_csv(icmp_file, chunksize=500_000,
                          usecols=["otime", "rtime", "ttime", "rtt_ms", "success"]):
    chunk = chunk[chunk["success"] == 1]
    t1 = chunk["otime"].astype("int64")
    t2 = chunk["rtime"].astype("int64")
    t3 = chunk["ttime"].astype("int64")
    t4 = t1 + chunk["rtt_ms"].astype("int64")
    offset = ((t2 - t1) + (t3 - t4)) // 2
    total_icmp += len(chunk)
    over_icmp += (offset.abs() > THRESHOLD_MS).sum()

pct_icmp = 100 * over_icmp / total_icmp if total_icmp else float("nan")
print(f"ICMP scan  ({icmp_file.name})")
print(f"  Total successful responses: {total_icmp:,}")
print(f"  |offset| > 5s:             {over_icmp:,}  ({pct_icmp:.2f}%)")
print()

# ── HTTP scans ────────────────────────────────────────────────────────────────
for label, path in [
    ("HTTP scan (icmp_with_http.csv)", ROOT / "data/icmp_with_http.csv"),
    ("HTTP scan (tranco_http.csv)",    ROOT / "data/tranco_http.csv"),
]:
    df = pd.read_csv(path, usecols=["http_clock_offset_ms"])
    df = df.dropna(subset=["http_clock_offset_ms"])
    total = len(df)
    over  = (df["http_clock_offset_ms"].abs() > THRESHOLD_MS).sum()
    pct   = 100 * over / total if total else float("nan")
    print(f"{label}")
    print(f"  Total hosts with HTTP offset: {total:,}")
    print(f"  |offset| > 5s:               {over:,}  ({pct:.2f}%)")
    print()
