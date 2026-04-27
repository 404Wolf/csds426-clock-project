import argparse

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
    parser = argparse.ArgumentParser(description="Outlier count by country")
    parser.add_argument("input", help="Path to enriched CSV (e.g. data/icmp_with_http.csv)")
    parser.add_argument("--min-hosts", type=int, default=5, help="Minimum hosts per country (default: 5)")
    parser.add_argument(
        "--sort",
        choices=["outliers", "pct_bad", "median"],
        default="outliers",
        help="Sort column (default: outliers)",
    )
    parser.add_argument("--threshold-ms", type=float, default=5000, help="Outlier threshold in ms (default: 5000)")
    args = parser.parse_args()

    df = pd.read_csv(args.input)
    df = df[(df["icmp_rtt_ms"] > 0) & (df["icmp_rtt_ms"] <= 10000)]
    df = df.dropna(subset=["country"])
    df["abs_offset"] = df["icmp_clock_offset_ms"].abs()

    thr = args.threshold_ms
    grouped = df.groupby("country").agg(
        hosts=("abs_offset", "count"),
        median_offset_ms=("abs_offset", "median"),
        n_outliers=("abs_offset", lambda x: int((x > thr).sum())),
        pct_bad=("abs_offset", lambda x: 100.0 * (x > thr).sum() / len(x)),
    ).reset_index()

    grouped = grouped[grouped["hosts"] >= args.min_hosts]

    sort_key = {
        "outliers": ("n_outliers", False),
        "pct_bad": ("pct_bad", False),
        "median": ("median_offset_ms", False),
    }[args.sort]
    grouped = grouped.sort_values(sort_key[0], ascending=sort_key[1]).reset_index(drop=True)

    col_widths = [3, 30, 7, 15, 10, 7]
    header = (
        f"{'#':>{col_widths[0]}}  {'Country':<{col_widths[1]}}  "
        f"{'Hosts':>{col_widths[2]}}  {'Median Offset':>{col_widths[3]}}  "
        f"{'# Outliers':>{col_widths[4]}}  {'% > 5s':>{col_widths[5]}}"
    )
    sep = "-" * len(header)
    thr_fmt = fmt_offset(thr)
    print(f"Outliers by country (sorted by {args.sort}, threshold >{thr_fmt}, min {args.min_hosts} hosts)")
    print(sep)
    print(header)
    print(sep)
    for i, row in grouped.iterrows():
        print(
            f"{i + 1:>{col_widths[0]}}  "
            f"{row['country']:<{col_widths[1]}}  "
            f"{int(row['hosts']):>{col_widths[2]}}  "
            f"{fmt_offset(row['median_offset_ms']):>{col_widths[3]}}  "
            f"{int(row['n_outliers']):>{col_widths[4]}}  "
            f"{row['pct_bad']:>{col_widths[5] - 1}.1f}%"
        )
    print(sep)
    top = grouped.iloc[0]
    print(f"Total countries: {len(grouped)}  |  Total hosts: {grouped['hosts'].sum()}")
    print(f"Most outliers: {top['country']} ({int(top['n_outliers'])} outliers, {top['pct_bad']:.1f}%)")


if __name__ == "__main__":
    main()
