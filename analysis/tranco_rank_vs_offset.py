import argparse
from pathlib import Path

import pandas as pd
import plotly.graph_objects as go


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Plot Tranco rank vs HTTP clock offset"
    )
    parser.add_argument("input", help="Path to enrich-http output CSV")
    parser.add_argument("tranco", help="Path to Tranco list CSV (rank,domain, no header)")
    parser.add_argument("-o", "--output", default="out/tranco_rank_vs_offset.html")
    parser.add_argument("--svg", action="store_true")
    args = parser.parse_args()

    df = pd.read_csv(args.input)
    tranco = pd.read_csv(args.tranco, header=None, names=["rank", "domain"])

    df = df.merge(tranco, left_on="hostname", right_on="domain", how="inner")
    df = df[df["is_frozen_clock"] == False]  # noqa: E712
    df = df[df["http_clock_offset_ms"].notna()]

    # IQR outlier removal on offset
    q1, q3 = df["http_clock_offset_ms"].quantile(0.25), df["http_clock_offset_ms"].quantile(0.75)
    iqr = q3 - q1
    df = df[df["http_clock_offset_ms"].between(q1 - 1.5 * iqr, q3 + 1.5 * iqr)]

    df = df.sort_values("rank")
    print(f"{len(df)} points after filtering")

    # Bin by rank and compute median + p25/p75 per bin
    BINS = 100
    df["bin"] = pd.cut(df["rank"], bins=BINS)
    binned = df.groupby("bin", observed=True)["http_clock_offset_ms"].agg(
        median="median", p25=lambda s: s.quantile(0.25), p75=lambda s: s.quantile(0.75)
    ).reset_index()
    binned["rank_mid"] = binned["bin"].apply(lambda b: b.mid)
    binned = binned.dropna()

    fig = go.Figure()

    # p25–p75 band
    fig.add_trace(go.Scatter(
        x=list(binned["rank_mid"]) + list(binned["rank_mid"])[::-1],
        y=list(binned["p75"]) + list(binned["p25"])[::-1],
        fill="toself",
        fillcolor="rgba(70,130,180,0.2)",
        line=dict(width=0),
        name="p25–p75",
        hoverinfo="skip",
    ))

    # Median line
    fig.add_trace(go.Scatter(
        x=binned["rank_mid"],
        y=binned["median"],
        mode="lines",
        line=dict(color="steelblue", width=2),
        name="median",
    ))

    fig.update_layout(
        title="Tranco Rank vs HTTP Clock Offset",
        xaxis=dict(title="Tranco Rank"),
        yaxis=dict(title="HTTP Clock Offset (ms)"),
        margin=dict(l=60, r=20, t=50, b=60),
        legend=dict(x=0.01, y=0.99),
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
