#!/usr/bin/env python3
"""Pretty-print a probe CSV with human-readable timestamps."""

import csv
import sys
from datetime import datetime, timezone
from itertools import groupby

def fmt_us(unix_us: int) -> str:
    dt = datetime.fromtimestamp(unix_us / 1_000_000, tz=timezone.utc)
    return dt.strftime("%H:%M:%S.") + f"{unix_us % 1_000_000:06d}"

def fmt_s(unix_s: int) -> str:
    dt = datetime.fromtimestamp(unix_s, tz=timezone.utc)
    return dt.strftime("%H:%M:%S")

path = sys.argv[1] if len(sys.argv) > 1 else "probes.csv"

with open(path) as f:
    rows = list(csv.DictReader(f))

# sort within each round by server second then offset, so the boundary reads cleanly
rows.sort(key=lambda r: (int(r["round"]), int(r["server_unix_s"]), int(r["offset_micros"])))

# reassign request numbers and compute deltas after re-sort
out_rows = []
for round_num, group in groupby(rows, key=lambda r: r["round"]):
    prev_send = None
    prev_recv = None
    for i, row in enumerate(group, 1):
        send = int(row["send_at_us"])
        recv = int(row["receive_at_us"])
        delta_send = "" if prev_send is None else str(send - prev_send)
        delta_recv = "" if prev_recv is None else str(recv - prev_recv)
        out_rows.append({
            "round": round_num,
            "request": str(i),
            "offset_micros": row["offset_micros"],
            "send_at": fmt_us(send),
            "receive_at": fmt_us(recv),
            "delta_send_us": delta_send,
            "delta_recv_us": delta_recv,
            "rtt_us": row["rtt_us"],
            "server_s": fmt_s(int(row["server_unix_s"])),
        })
        prev_send = send
        prev_recv = recv

fields = ["round", "request", "offset_micros", "send_at", "receive_at", "delta_send_us", "delta_recv_us", "rtt_us", "server_s"]
print(",".join(fields))
for r in out_rows:
    print(",".join(r[f] for f in fields))
