"""Scatter plot comparing ICMP vs HTTP clock offset measurements."""

import argparse
from pathlib import Path

import pandas as pd
import plotly.express as px


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Scatter plot of ICMP vs HTTP clock offsets"
    )
    parser.add_argument("input", help="Path to enrich-http output CSV")
    parser.add_argument(
        "-o", "--output", default="icmp_vs_http.html", help="Output file path"
    )
    args = parser.parse_args()

    df = pd.read_csv(args.input)
    with pd.option_context("display.max_rows", None, "display.max_columns", None):
        print(df)

    # IQR-based outlier removal on both offset axes
    for col in ("icmp_clock_offset_ms", "http_clock_offset_ms"):
        q1, q3 = df[col].quantile(0.25), df[col].quantile(0.75)
        iqr = q3 - q1
        df = df[df[col].between(q1 - 1.5 * iqr, q3 + 1.5 * iqr)]
    print(f"{len(df)} points after outlier removal")

    fig = px.scatter(
        df,
        x="icmp_clock_offset_ms",
        y="http_clock_offset_ms",
        hover_data=["ip", "hostname", "icmp_rtt_ms", "http_rtt_us", "country", "city"],
        labels={
            "icmp_clock_offset_ms": "ICMP Clock Offset (ms)",
            "http_clock_offset_ms": "HTTP Clock Offset (ms)",
            "icmp_rtt_ms": "ICMP RTT (ms)",
            "http_rtt_us": "HTTP RTT (µs)",
        },
        title="ICMP vs HTTP Clock Offset",
    )

    # y = x reference line
    all_vals = pd.concat([df["icmp_clock_offset_ms"], df["http_clock_offset_ms"]])
    lo, hi = all_vals.min(), all_vals.max()
    pad = (hi - lo) * 0.05 if hi != lo else 1
    fig.add_shape(
        type="line",
        x0=lo - pad,
        y0=lo - pad,
        x1=hi + pad,
        y1=hi + pad,
        line=dict(color="gray", dash="dash", width=1),
    )

    fig.update_traces(marker=dict(size=10, line=dict(width=1, color="black")))
    fig.update_layout(
        xaxis=dict(scaleanchor="y", scaleratio=1),
        margin=dict(l=60, r=20, t=50, b=60),
    )

    output = Path(args.output)
    if output.suffix in (".png", ".svg", ".pdf"):
        fig.write_image(str(output), width=900, height=900, scale=2)
    else:
        fig.write_html(str(output))
    print(f"Wrote {output}")


if __name__ == "__main__":
    main()
