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
        "-o", "--output", default="out/icmp_vs_http.html", help="Output file path"
    )
    parser.add_argument("--svg", action="store_true", help="Also save an SVG")
    parser.add_argument("--outliers", type=float, metavar="SECONDS",
                        help="Keep only points with |offset| < SECONDS on both axes (omit for IQR removal)")
    args = parser.parse_args()

    df = pd.read_csv(args.input)

    if args.outliers is not None:
        threshold_ms = args.outliers * 1000
        for col in ("icmp_clock_offset_ms", "http_clock_offset_ms"):
            df = df[df[col].abs() < threshold_ms]
    else:
        # IQR-based outlier removal on both offset axes
        for col in ("icmp_clock_offset_ms", "http_clock_offset_ms"):
            q1, q3 = df[col].quantile(0.25), df[col].quantile(0.75)
            iqr = q3 - q1
            df = df[df[col].between(q1 - 1.5 * iqr, q3 + 1.5 * iqr)]
    print(f"{len(df)} points")

    fig = px.scatter(
        df,
        x="icmp_clock_offset_ms",
        y="http_clock_offset_ms",
        hover_data=["ip", "hostname", "icmp_rtt_ms", "country", "city"],
        labels={
            "icmp_clock_offset_ms": "ICMP Clock Offset (ms)",
            "http_clock_offset_ms": "HTTP Clock Offset (ms)",
            "icmp_rtt_ms": "ICMP RTT (ms)",
        },
        title="ICMP vs HTTP Clock Offset",
    )

    fig.update_traces(marker=dict(size=4, line=dict(width=1, color="black")))
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

    if args.svg:
        svg_out = output.with_suffix(".svg")
        fig.write_image(str(svg_out))
        print(f"Wrote {svg_out}")


if __name__ == "__main__":
    main()
