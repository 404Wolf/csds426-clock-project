#!/usr/bin/env python3
"""Print % of hosts with clocks >5 seconds off from ICMP and HTTP scans."""

import pandas as pd
from pathlib import Path

ROOT = Path(__file__).parent.parent
THRESHOLD_MS = 5_000  # 5 seconds

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

# HTTP + ICMP: report both HTTP and ICMP >5s rates
df = pd.read_csv(ROOT / "data/icmp_with_http.csv",
                 usecols=["http_clock_offset_ms", "icmp_clock_offset_ms"])
df_http = df.dropna(subset=["http_clock_offset_ms"])
total_http = len(df_http)
over_http  = (df_http["http_clock_offset_ms"].abs() > THRESHOLD_MS).sum()
pct_http   = 100 * over_http / total_http if total_http else float("nan")

df_icmp = df.dropna(subset=["icmp_clock_offset_ms"])
total_icmp2 = len(df_icmp)
over_icmp2  = (df_icmp["icmp_clock_offset_ms"].abs() > THRESHOLD_MS).sum()
pct_icmp2   = 100 * over_icmp2 / total_icmp2 if total_icmp2 else float("nan")

print("HTTP + ICMP scan (icmp_with_http.csv)")
print(f"  Hosts with HTTP offset: {total_http:,}  |offset| > 5s: {over_http:,}  ({pct_http:.2f}%)")
print(f"  Hosts with ICMP offset: {total_icmp2:,}  |offset| > 5s: {over_icmp2:,}  ({pct_icmp2:.2f}%)")
print()

# Tranco: HTTP only
df = pd.read_csv(ROOT / "data/tranco_http.csv", usecols=["http_clock_offset_ms"])
df = df.dropna(subset=["http_clock_offset_ms"])
total = len(df)
over  = (df["http_clock_offset_ms"].abs() > THRESHOLD_MS).sum()
pct   = 100 * over / total if total else float("nan")
print("Tranco (tranco_http.csv)")
print(f"  Total hosts with HTTP offset: {total:,}")
print(f"  |offset| > 5s:               {over:,}  ({pct:.2f}%)")
print()
