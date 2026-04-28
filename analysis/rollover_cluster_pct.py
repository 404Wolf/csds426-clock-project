#!/usr/bin/env python3
"""
Among ICMP hosts with the nonstandard-timestamp bit set, find what fraction
cluster near the 32-bit millisecond rollover point (2^32 ms ≈ 49.7 days).

These hosts are excluded from clock-sync analysis (per methodology), but their
offsets reveal an interesting artifact: many appear to be returning a raw
uptime counter rather than a wall-clock timestamp.
"""

import pandas as pd
from pathlib import Path

ROOT = Path(__file__).parent.parent

# 2^32 ms = 4,294,967,296 ms ≈ 49.71 days
ROLLOVER_MS = 2**32
# Window: [46.3, 49.7) days
WINDOW_LOW_MS  = 4_000_000_000
WINDOW_HIGH_MS = ROLLOVER_MS

icmp_file = ROOT / "data/icmp_timestamp/icmp_timestamp.csv"

total_ns = 0           # nonstandard hosts
in_window = 0          # nonstandard hosts whose offset falls in rollover window

for chunk in pd.read_csv(
    icmp_file,
    chunksize=500_000,
    usecols=["otime", "rtime", "ttime", "rtt_ms", "success", "ts_nonstandard"],
):
    chunk = chunk[(chunk["success"] == 1) & (chunk["ts_nonstandard"] == 1)]
    if chunk.empty:
        continue

    t1 = chunk["otime"].astype("int64")
    t2 = chunk["rtime"].astype("int64")
    t3 = chunk["ttime"].astype("int64")
    t4 = t1 + chunk["rtt_ms"].astype("int64")
    offset = ((t2 - t1) + (t3 - t4)) // 2

    total_ns += len(chunk)
    in_window += ((offset >= WINDOW_LOW_MS) & (offset < WINDOW_HIGH_MS)).sum()

pct = 100 * in_window / total_ns if total_ns else float("nan")
days_low  = WINDOW_LOW_MS  / 86_400_000
days_high = WINDOW_HIGH_MS / 86_400_000

print(f"Nonstandard-timestamp ICMP hosts (ts_nonstandard=1):")
print(f"  Total:                           {total_ns:>12,}")
print(f"  In rollover window [{days_low:.1f}–{days_high:.1f} days]: {in_window:>12,}  ({pct:.1f}%)")
