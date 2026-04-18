import argparse
from pathlib import Path

import pandas as pd
import plotly.graph_objects as go


def load_responding_ips(path: str) -> set[int]:
    """Load CSV and return the set of saddr_raw values where success==1."""
    df = pd.read_csv(path, usecols=["saddr_raw", "success"],
                     dtype={"saddr_raw": "int64", "success": "int8"})
    return set(df.loc[df["success"] == 1, "saddr_raw"].tolist())


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
    ts_ips = load_responding_ips(args.timestamp)
    print(f"  {len(ts_ips):,} IPs responded to ICMP timestamp")

    print("Loading echo-responding IPs...")
    echo_ips = load_responding_ips(args.echo)
    print(f"  {len(echo_ips):,} IPs responded to ICMP echo (ping)")

    both = echo_ips & ts_ips
    echo_only = echo_ips - ts_ips
    ts_only = ts_ips - echo_ips

    print(f"\nOverlap: {len(both):,} IPs respond to both")
    print(f"Echo only: {len(echo_only):,}")
    print(f"Timestamp only: {len(ts_only):,}")
    print(f"Of ping servers, {100*len(both)/len(echo_ips):.1f}% also do timestamp")

    categories = ["Echo (ping) only", "Both", "Timestamp only"]
    counts = [len(echo_only), len(both), len(ts_only)]
    colors = ["steelblue", "mediumseagreen", "crimson"]

    fig = go.Figure(go.Bar(
        x=categories,
        y=counts,
        marker_color=colors,
        text=[f"{c:,}<br>({100*c/sum(counts):.1f}%)" for c in counts],
        textposition="outside",
    ))

    pct = 100 * len(both) / len(echo_ips)
    fig.update_layout(
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
