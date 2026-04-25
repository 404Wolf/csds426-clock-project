import argparse
from pathlib import Path

import pandas as pd
import plotly.graph_objects as go


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Plot Tranco rank vs HTTP clock offset"
    )
    parser.add_argument("input", help="Path to tranco_1k.csv")
    parser.add_argument("tranco", help="Path to Tranco 1M list CSV (rank,domain, no header)")
    parser.add_argument("-o", "--output", default="out/tranco_rank_vs_offset.html")
    parser.add_argument("--svg", action="store_true")
    args = parser.parse_args()

    df = pd.read_csv(args.input)
    tranco = pd.read_csv(args.tranco, header=None, names=["rank", "domain"])

    df = df.merge(tranco, left_on="hostname", right_on="domain", how="inner")
    df = df.rename(columns={"clock_offset_ms": "http_clock_offset_ms"})
    df = df[df["http_clock_offset_ms"].notna()]

    # IQR outlier removal on offset
    q1, q3 = df["http_clock_offset_ms"].quantile(0.25), df["http_clock_offset_ms"].quantile(0.75)
    iqr = q3 - q1
    df = df[df["http_clock_offset_ms"].between(q1 - 1.5 * iqr, q3 + 1.5 * iqr)]

    df = df.sort_values("rank")
    print(f"{len(df)} points after filtering, rank range {df['rank'].min()}–{df['rank'].max()}")

    fig = go.Figure(go.Scatter(
        x=df["rank"],
        y=df["http_clock_offset_ms"],
        mode="lines+markers",
        marker=dict(size=4, color="steelblue"),
        line=dict(color="steelblue", width=1),
        name="clock offset",
    ))

    fig.update_layout(
        title="Tranco Rank vs HTTP Clock Offset",
        xaxis=dict(title="Tranco Rank"),
        yaxis=dict(title="HTTP Clock Offset (ms)"),
        margin=dict(l=60, r=20, t=50, b=60),
    )

    output = Path(args.output)
    if output.suffix in (".png", ".svg", ".pdf"):
        fig.write_image(str(output), width=1200, height=600, scale=2)
    else:
        fig.write_html(str(output))
    print(f"Wrote {output}")

    if args.svg:
        svg_out = output.with_suffix(".svg")
        fig.write_image(str(svg_out), width=1200, height=600, scale=2)
        print(f"Wrote {svg_out}")


if __name__ == "__main__":
    main()
