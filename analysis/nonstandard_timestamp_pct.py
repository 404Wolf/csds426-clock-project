#!/usr/bin/env python3
"""% of ICMP timestamp hosts that flip the high-order bit (ts_nonstandard=1)."""

import pandas as pd
from pathlib import Path

ROOT = Path(__file__).parent.parent
icmp_file = ROOT / "data/icmp_timestamp/icmp_timestamp.csv"

hosts_seen: set = set()
hosts_with_standard: set = set()  # hosts that sent at least one standard timestamp
hosts_with_nonstandard: set = set()  # hosts that sent at least one nonstandard timestamp

for chunk in pd.read_csv(icmp_file, chunksize=500_000,
                         usecols=["saddr", "ts_nonstandard", "success"]):
    chunk = chunk[chunk["success"] == 1]
    hosts_seen.update(chunk["saddr"])
    hosts_with_standard.update(chunk.loc[chunk["ts_nonstandard"] == 0, "saddr"])
    hosts_with_nonstandard.update(chunk.loc[chunk["ts_nonstandard"] == 1, "saddr"])

total_hosts = len(hosts_seen)
n_any = len(hosts_with_nonstandard)
n_all = len(hosts_seen - hosts_with_standard)  # never sent a standard timestamp

print(f"ICMP timestamp scan  ({icmp_file.name})")
print(f"  Unique hosts (successful responses): {total_hosts:,}")
print(f"  Hosts with ANY nonstandard timestamp:  {n_any:,}  ({100*n_any/total_hosts:.2f}%)")
print(f"  Hosts with ALL nonstandard timestamps: {n_all:,}  ({100*n_all/total_hosts:.2f}%)")
