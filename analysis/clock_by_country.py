import argparse
import math

import pandas as pd


def fmt_offset(ms: float) -> str:
    ms = abs(ms)
    if ms < 1000:
        return f"{ms:.0f}ms"
    if ms < 60_000:
        return f"{ms / 1000:.1f}s"
    if ms < 3_600_000:
        return f"{ms / 60_000:.1f}min"
    if ms < 86_400_000:
        return f"{ms / 3_600_000:.1f}hr"
    return f"{ms / 86_400_000:.1f}days"


def main() -> None:
    parser = argparse.ArgumentParser(description="List clock accuracy by country")
    parser.add_argument("input", help="Path to enriched CSV (e.g. data/icmp_with_http.csv)")
    parser.add_argument("--min-hosts", type=int, default=5, help="Minimum hosts per country (default: 5)")
    args = parser.parse_args()

    df = pd.read_csv(args.input)
    df = df[(df["icmp_rtt_ms"] > 0) & (df["icmp_rtt_ms"] <= 10000)]
    df = df.dropna(subset=["country"])
    df["abs_offset"] = df["icmp_clock_offset_ms"].abs()

    grouped = df.groupby("country").agg(
        hosts=("abs_offset", "count"),
        median_offset_ms=("abs_offset", "median"),
        pct_bad=("abs_offset", lambda x: 100.0 * (x > 5000).sum() / len(x)),
    ).reset_index()

    grouped = grouped[grouped["hosts"] >= args.min_hosts]
    grouped = grouped.sort_values("median_offset_ms").reset_index(drop=True)

    col_widths = [3, 30, 7, 15, 7]
    header = f"{'#':>{col_widths[0]}}  {'Country':<{col_widths[1]}}  {'Hosts':>{col_widths[2]}}  {'Median Offset':>{col_widths[3]}}  {'% > 5s':>{col_widths[4]}}"
    sep = "-" * len(header)
    print(f"Clock accuracy by country (sorted best first, min {args.min_hosts} hosts)")
    print(sep)
    print(header)
    print(sep)
    for i, row in grouped.iterrows():
        print(
            f"{i + 1:>{col_widths[0]}}  "
            f"{row['country']:<{col_widths[1]}}  "
            f"{int(row['hosts']):>{col_widths[2]}}  "
            f"{fmt_offset(row['median_offset_ms']):>{col_widths[3]}}  "
            f"{row['pct_bad']:>{col_widths[4] - 1}.1f}%"
        )
    print(sep)
    print(f"Total countries: {len(grouped)}  |  Total hosts: {grouped['hosts'].sum()}")


if __name__ == "__main__":
    main()
