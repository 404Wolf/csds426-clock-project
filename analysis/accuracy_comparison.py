import argparse
from pathlib import Path

import numpy as np
import pandas as pd
import plotly.graph_objects as go
from plotly.subplots import make_subplots


def cdf(series: pd.Series):
    s = series.sort_values()
    return s.values, np.linspace(0, 1, len(s))


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Compare ICMP vs HTTP clock offset accuracy"
    )
    parser.add_argument("input", help="CSV with both icmp_clock_offset_ms and http_clock_offset_ms")
    parser.add_argument("-o", "--output", default="out/accuracy_comparison.html")
    parser.add_argument("--outliers", type=float, metavar="SECONDS",
                        help="Keep only |offset| < SECONDS (omit for IQR removal)")
    parser.add_argument("--svg", action="store_true")
    args = parser.parse_args()

    df = pd.read_csv(args.input)
    df = df[df["icmp_clock_offset_ms"].notna() & df["http_clock_offset_ms"].notna()]

    if args.outliers is not None:
        t = args.outliers * 1000
        df = df[df["icmp_clock_offset_ms"].abs() < t]
        df = df[df["http_clock_offset_ms"].abs() < t]
    else:
        for col in ("icmp_clock_offset_ms", "http_clock_offset_ms"):
            q1, q3 = df[col].quantile(0.25), df[col].quantile(0.75)
            iqr = q3 - q1
            df = df[df[col].between(q1 - 1.5 * iqr, q3 + 1.5 * iqr)]

    print(f"{len(df)} hosts with both measurements")

    icmp = df["icmp_clock_offset_ms"]
    http = df["http_clock_offset_ms"]

    fig = make_subplots(
        rows=1, cols=2,
        subplot_titles=(
            "CDF of |Clock Offset|",
            "Clock Offset Distribution",
        ),
    )

    # Left: CDF of absolute offsets
    for series, name, color in [
        (icmp.abs(), "ICMP", "steelblue"),
        (http.abs(), "HTTP", "crimson"),
    ]:
        x, y = cdf(series)
        fig.add_trace(go.Scatter(
            x=x, y=y, mode="lines", name=name,
            line=dict(color=color, width=2),
        ), row=1, col=1)

    # Right: overlapping KDE-style histogram of signed offsets
    for series, name, color in [
        (icmp, "ICMP", "steelblue"),
        (http, "HTTP", "crimson"),
    ]:
        fig.add_trace(go.Histogram(
            x=series, name=name,
            marker_color=color, opacity=0.5,
            histnorm="probability density",
            showlegend=False,
            nbinsx=80,
        ), row=1, col=2)

    fig.update_xaxes(title_text="|Offset| (ms)", row=1, col=1)
    fig.update_yaxes(title_text="Fraction of hosts", row=1, col=1)
    fig.update_xaxes(title_text="Offset (ms)", row=1, col=2)
    fig.update_yaxes(title_text="Density", row=1, col=2)

    fig.update_layout(
        title="ICMP vs HTTP Clock Offset Accuracy",
        barmode="overlay",
        legend=dict(x=0.38, y=0.05),
        margin=dict(l=60, r=20, t=60, b=60),
        width=1200,
        height=500,
    )

    output = Path(args.output)
    if output.suffix in (".png", ".svg", ".pdf"):
        fig.write_image(str(output), width=1200, height=500, scale=2)
    else:
        fig.write_html(str(output))
    print(f"Wrote {output}")

    if args.svg:
        svg_out = output.with_suffix(".svg")
        fig.write_image(str(svg_out), width=1200, height=500, scale=2)
        print(f"Wrote {svg_out}")

    for label, s in [("ICMP", icmp), ("HTTP", http)]:
        print(f"{label}: median={s.median():.1f}ms  std={s.std():.1f}ms  "
              f"p90={s.abs().quantile(0.9):.1f}ms  p99={s.abs().quantile(0.99):.1f}ms")


if __name__ == "__main__":
    main()
