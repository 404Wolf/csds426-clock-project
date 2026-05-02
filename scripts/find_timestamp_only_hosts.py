#!/usr/bin/env python3
"""
Find hosts that respond to ICMP Timestamp (type 13) but NOT ICMP Echo (type 8/ping).

Must be run as root (raw socket requires CAP_NET_RAW).
"""

import argparse
import array
import csv
import ipaddress
import os
import resource
import select
import socket
import struct
import sys
import time
from dataclasses import dataclass, field
from typing import Iterator


# ---------------------------------------------------------------------------
# Memory hard-limit (set before anything else allocates)
# ---------------------------------------------------------------------------

def enforce_memory_limit(max_bytes: int) -> None:
    soft, hard = resource.getrlimit(resource.RLIMIT_AS)
    new_limit = min(max_bytes, hard) if hard != resource.RLIM_INFINITY else max_bytes
    resource.setrlimit(resource.RLIMIT_AS, (new_limit, hard))


# ---------------------------------------------------------------------------
# ICMP packet helpers
# ---------------------------------------------------------------------------

ICMP_ECHO_REQUEST   = 8
ICMP_ECHO_REPLY     = 0
ICMP_TIMESTAMP_REQUEST = 13
ICMP_TIMESTAMP_REPLY   = 14

ICMP_ECHO_SIZE      = 8   # header only, no data payload
ICMP_TIMESTAMP_SIZE = 20  # header (8) + 3 × 32-bit timestamps


def _checksum(data: bytes) -> int:
    """RFC 1071 Internet checksum."""
    if len(data) % 2:
        data += b"\x00"
    arr = array.array("H", data)
    total = sum(arr)
    total = (total >> 16) + (total & 0xFFFF)
    total += total >> 16
    return ~total & 0xFFFF


def build_echo(ident: int, seq: int) -> bytes:
    header = struct.pack("!BBHHH", ICMP_ECHO_REQUEST, 0, 0, ident, seq)
    csum   = _checksum(header)
    return struct.pack("!BBHHH", ICMP_ECHO_REQUEST, 0, csum, ident, seq)


def build_timestamp(ident: int, seq: int) -> bytes:
    ms_since_midnight = int((time.time() % 86400) * 1000) & 0xFFFFFFFF
    header = struct.pack("!BBHHH", ICMP_TIMESTAMP_REQUEST, 0, 0, ident, seq)
    body   = struct.pack("!III", ms_since_midnight, 0, 0)
    raw    = header + body
    csum   = _checksum(raw)
    return struct.pack("!BBHHH", ICMP_TIMESTAMP_REQUEST, 0, csum, ident, seq) + body


# ---------------------------------------------------------------------------
# Probing logic
# ---------------------------------------------------------------------------

TIMEOUT = 2.0        # seconds to wait for replies per round
BUF_SIZE = 65536


@dataclass
class ProbeResult:
    ip: str
    echo_reply: bool   = False
    ts_reply:   bool   = False


def _recv_replies(
    sock: socket.socket,
    deadline: float,
    ident: int,
    results: dict[str, ProbeResult],
) -> None:
    while True:
        now = time.monotonic()
        if now >= deadline:
            break
        ready, _, _ = select.select([sock], [], [], deadline - now)
        if not ready:
            break
        try:
            pkt, addr = sock.recvfrom(BUF_SIZE)
        except OSError:
            continue
        ip = addr[0]
        if ip not in results:
            continue
        # IP header is at least 20 bytes; ICMP starts after it
        ip_hlen = (pkt[0] & 0x0F) * 4
        if len(pkt) < ip_hlen + 2:
            continue
        icmp_type = pkt[ip_hlen]
        if icmp_type == ICMP_ECHO_REPLY:
            # Verify ident
            if len(pkt) >= ip_hlen + 8:
                recv_ident = struct.unpack_from("!H", pkt, ip_hlen + 4)[0]
                if recv_ident == ident:
                    results[ip].echo_reply = True
        elif icmp_type == ICMP_TIMESTAMP_REPLY:
            if len(pkt) >= ip_hlen + 8:
                recv_ident = struct.unpack_from("!H", pkt, ip_hlen + 4)[0]
                if recv_ident == ident:
                    results[ip].ts_reply = True


def probe_batch(
    ips: list[str],
    ident: int,
    timeout: float = TIMEOUT,
) -> list[ProbeResult]:
    """
    Probe a batch of IPs. Returns one ProbeResult per IP.
    Uses a single raw ICMP socket (requires root / CAP_NET_RAW).
    """
    results: dict[str, ProbeResult] = {ip: ProbeResult(ip=ip) for ip in ips}

    with socket.socket(
        socket.AF_INET, socket.SOCK_RAW, socket.IPPROTO_ICMP
    ) as sock:
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_RCVBUF, 4 * 1024 * 1024)

        # --- send echo requests ---
        for seq, ip in enumerate(ips):
            try:
                sock.sendto(build_echo(ident, seq & 0xFFFF), (ip, 0))
            except OSError:
                pass

        deadline = time.monotonic() + timeout
        _recv_replies(sock, deadline, ident, results)

        # --- send timestamp requests ---
        for seq, ip in enumerate(ips):
            try:
                sock.sendto(build_timestamp(ident, seq & 0xFFFF), (ip, 0))
            except OSError:
                pass

        deadline = time.monotonic() + timeout
        _recv_replies(sock, deadline, ident, results)

    return list(results.values())


# ---------------------------------------------------------------------------
# IP source helpers
# ---------------------------------------------------------------------------

def ips_from_file(path: str) -> Iterator[str]:
    """Yield IP addresses from a file (one per line, or CSV column)."""
    with open(path) as fh:
        first = fh.readline().strip()
        fh.seek(0)
        # Detect CSV: if the first line has commas try column 2 (index 1)
        if "," in first:
            reader = csv.reader(fh)
            next(reader, None)  # skip header
            for row in reader:
                if len(row) >= 2:
                    ip = row[1].strip()
                    try:
                        ipaddress.ip_address(ip)
                        yield ip
                    except ValueError:
                        pass
        else:
            for line in fh:
                ip = line.strip()
                if not ip or ip.startswith("#"):
                    continue
                try:
                    ipaddress.ip_address(ip)
                    yield ip
                except ValueError:
                    pass


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description="Find hosts responding to ICMP Timestamp but NOT ICMP Echo."
    )
    p.add_argument("input", help="File of IPs (plain list or CSV with IP in column 2)")
    p.add_argument(
        "-o", "--output", default="-",
        help="Output file for results (default: stdout)"
    )
    p.add_argument(
        "-b", "--batch-size", type=int, default=256,
        help="IPs per probe batch (default: 256)"
    )
    p.add_argument(
        "-t", "--timeout", type=float, default=TIMEOUT,
        help=f"Reply wait timeout in seconds (default: {TIMEOUT})"
    )
    p.add_argument(
        "--memory-limit-gb", type=float, default=10.0,
        help="Virtual memory limit in GB (default: 10)"
    )
    return p.parse_args()


def batched(it: Iterator[str], size: int) -> Iterator[list[str]]:
    batch: list[str] = []
    for item in it:
        batch.append(item)
        if len(batch) == size:
            yield batch
            batch = []
    if batch:
        yield batch


def main() -> None:
    args = parse_args()

    # Enforce memory limit before any significant allocation
    limit_bytes = int(args.memory_limit_gb * 1024 ** 3)
    enforce_memory_limit(limit_bytes)

    if os.geteuid() != 0:
        sys.exit("Error: raw ICMP sockets require root (or CAP_NET_RAW).")

    ident = os.getpid() & 0xFFFF
    found: list[str] = []

    out_fh = open(args.output, "w") if args.output != "-" else sys.stdout
    try:
        writer = csv.writer(out_fh)
        writer.writerow(["ip", "echo_reply", "ts_reply", "ts_only"])

        total = 0
        ts_only_count = 0

        for batch in batched(ips_from_file(args.input), args.batch_size):
            results = probe_batch(batch, ident=ident, timeout=args.timeout)
            for r in results:
                total += 1
                ts_only = r.ts_reply and not r.echo_reply
                if ts_only:
                    ts_only_count += 1
                    found.append(r.ip)
                writer.writerow([r.ip, int(r.echo_reply), int(r.ts_reply), int(ts_only)])

        print(
            f"\n# Scanned: {total}  |  TS-only (no echo): {ts_only_count}",
            file=sys.stderr,
        )
        if found:
            print("# Timestamp-only hosts:", file=sys.stderr)
            for ip in found:
                print(f"#   {ip}", file=sys.stderr)
    finally:
        if out_fh is not sys.stdout:
            out_fh.close()


if __name__ == "__main__":
    main()
