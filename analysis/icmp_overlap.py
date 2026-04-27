import argparse
from pathlib import Path

import polars as pl
import plotly.graph_objects as go


def load_responding_ips(path: str) -> set[int]:
    """Return set of saddr_raw values where success==1."""
    df = (
        pl.scan_csv(path, schema_overrides={"saddr_raw": pl.Int64, "success": pl.Int8})
        .filter(pl.col("success") == 1)
        .select("saddr_raw")
        .collect()
    )
    return set(df["saddr_raw"].to_list())


def load_timestamp_ips(path: str) -> tuple[set[int], set[int]]:
    """Return (standard_ips, nonstandard_ips) for success==1 timestamp rows."""
    df = (
        pl.scan_csv(path, schema_overrides={"saddr_raw": pl.Int64, "success": pl.Int8, "ts_nonstandard": pl.Int8})
        .filter(pl.col("success") == 1)
        .select("saddr_raw", "ts_nonstandard")
        .collect()
    )
    standard = set(df.filter(pl.col("ts_nonstandard") == 0)["saddr_raw"].to_list())
    nonstandard = set(df.filter(pl.col("ts_nonstandard") == 1)["saddr_raw"].to_list())
    return standard, nonstandard


def main() -> None:
    parser = argparse.ArgumentParser(
        description="How often do ICMP ping servers also support ICMP timestamp?"
    )
    parser.add_argument("echo", help="icmp_echo CSV")
    parser.add_argument("timestamp", help="icmp_timestamp CSV")
    parser.add_argument("-o", "--output", default="out/icmp_overlap.html")
    parser.add_argument("--svg", action="store_true")
    args = parser.parse_args()

    print("Loading timestamp-responding IPs...")
    ts_standard, ts_nonstandard = load_timestamp_ips(args.timestamp)
    ts_ips = ts_standard | ts_nonstandard
    print(f"  {len(ts_ips):,} IPs responded to ICMP timestamp")
    print(f"    {len(ts_standard):,} standard (ts_nonstandard=0)")
    print(f"    {len(ts_nonstandard):,} non-standard (ts_nonstandard=1)")

    print("Loading echo-responding IPs...")
    echo_ips = load_responding_ips(args.echo)
    print(f"  {len(echo_ips):,} IPs responded to ICMP echo (ping)")

    both = echo_ips & ts_ips
    echo_only = echo_ips - ts_ips
    ts_only = ts_ips - echo_ips

    both_nonstandard = both & ts_nonstandard
    both_standard = both - both_nonstandard
    ts_only_nonstandard = ts_only & ts_nonstandard
    ts_only_standard = ts_only - ts_only_nonstandard

    print(f"\nOverlap: {len(both):,} IPs respond to both")
    print(f"Echo only: {len(echo_only):,}")
    print(f"Timestamp only: {len(ts_only):,}")
    print(f"Of ping servers, {100*len(both)/len(echo_ips):.1f}% also do timestamp")

    categories = ["Echo (ping) only", "Both", "Timestamp only"]
    total = len(echo_only) + len(both) + len(ts_only)

    fig = go.Figure()

    # Standard timestamp portion (or full bar for echo-only)
    standard_counts = [len(echo_only), len(both_standard), len(ts_only_standard)]
    fig.add_trace(go.Bar(
        name="Standard timestamp",
        x=categories,
        y=standard_counts,
        marker_color=["steelblue", "mediumseagreen", "crimson"],
        text=[f"{c:,}<br>({100*c/total:.1f}%)" for c in standard_counts],
        textposition="inside",
    ))

    # Non-standard timestamp portion
    nonstandard_counts = [0, len(both_nonstandard), len(ts_only_nonstandard)]
    fig.add_trace(go.Bar(
        name="Non-standard timestamp (ts_nonstandard=1)",
        x=categories,
        y=nonstandard_counts,
        marker_color=["rgba(0,0,0,0)", "rgba(255,165,0,0.7)", "rgba(255,165,0,0.7)"],
        text=[("" if c == 0 else f"{c:,}<br>({100*c/total:.1f}%)") for c in nonstandard_counts],
        textposition="inside",
    ))

    pct = 100 * len(both) / len(echo_ips)
    fig.update_layout(
        barmode="stack",
        title=f"ICMP Echo vs Timestamp Support — {pct:.1f}% of ping servers also support timestamp",
        yaxis_title="Number of IPs",
        margin=dict(l=60, r=20, t=70, b=60),
        width=700,
        height=500,
    )

    output = Path(args.output)
    if output.suffix in (".png", ".svg", ".pdf"):
        fig.write_image(str(output), width=700, height=500, scale=2)
    else:
        fig.write_html(str(output))
    print(f"\nWrote {output}")

    if args.svg:
        svg_out = output.with_suffix(".svg")
        fig.write_image(str(svg_out), width=700, height=500, scale=2)
        print(f"Wrote {svg_out}")


if __name__ == "__main__":
    main()
