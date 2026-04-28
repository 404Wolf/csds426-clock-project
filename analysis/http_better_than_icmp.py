#!/usr/bin/env python3

import pandas as pd
from pathlib import Path

ROOT = Path(__file__).parent.parent

df = pd.read_csv(
    ROOT / "data/icmp_with_http.csv",
    usecols=["ip", "hostname", "country", "city", "icmp_clock_offset_ms", "http_clock_offset_ms"],
)

df = df.dropna(subset=["icmp_clock_offset_ms", "http_clock_offset_ms"])
df["icmp_abs_ms"] = df["icmp_clock_offset_ms"].abs()
df["http_abs_ms"] = df["http_clock_offset_ms"].abs()
df["improvement_ms"] = df["icmp_abs_ms"] - df["http_abs_ms"]

better = df[df["http_abs_ms"] < df["icmp_abs_ms"]].copy()
better = better.sort_values("improvement_ms", ascending=False)

out = better[["ip", "hostname", "country", "city", "icmp_clock_offset_ms", "http_clock_offset_ms", "icmp_abs_ms", "http_abs_ms", "improvement_ms"]]
import sys
out.to_csv(sys.stdout, index=False)
