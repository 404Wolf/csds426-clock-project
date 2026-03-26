"""Plot ICMP clock sync data on a world map, colored by sync quality."""

import argparse
import math
from pathlib import Path

import pandas as pd
import plotly.express as px
import plotly.graph_objects as go


def load_and_filter(csv_path: str) -> pd.DataFrame:
    df = pd.read_csv(csv_path)
    df = df[(df["rtt_ms"] > 0) & (df["rtt_ms"] <= 10000)]
    df = df.dropna(subset=["latitude", "longitude"])
    return df


def compute_log_offset(df: pd.DataFrame) -> pd.DataFrame:
    df = df.copy()
    df["abs_offset_ms"] = df["clock_offset_ms"].abs()
    df["log_offset"] = df["abs_offset_ms"].apply(lambda x: math.log10(x + 1))
    return df


def build_figure(df: pd.DataFrame) -> go.Figure:
    # Precompute human-readable offset for hover
    def fmt_offset(ms: float) -> str:
        ms = abs(ms)
        if ms < 1000:
            return f"{ms:.0f}ms"
        elif ms < 60_000:
            return f"{ms / 1000:.1f}s"
        elif ms < 3_600_000:
            return f"{ms / 60_000:.1f}min"
        elif ms < 86_400_000:
            return f"{ms / 3_600_000:.1f}hr"
        else:
            return f"{ms / 86_400_000:.1f}days"

    df = df.copy()
    df["offset_label"] = df["clock_offset_ms"].apply(fmt_offset)
    df["hostname_display"] = df["hostname"].fillna("(none)")

    fig = px.scatter_geo(
        df,
        lat="latitude",
        lon="longitude",
        color="log_offset",
        color_continuous_scale="RdYlGn_r",
        range_color=[0, df["log_offset"].quantile(0.95)],
        projection="natural earth",
        hover_data={
            "log_offset": False,
            "latitude": ":.2f",
            "longitude": ":.2f",
            "offset_label": True,
            "rtt_ms": True,
            "ip": True,
            "hostname_display": True,
            "city": True,
            "country": True,
        },
        labels={
            "offset_label": "Clock Offset",
            "rtt_ms": "RTT (ms)",
            "hostname_display": "Hostname",
        },
    )

    # Custom colorbar ticks: map human-readable times to log10 values
    tick_map = {
        "0ms": 0,
        "10ms": math.log10(11),
        "100ms": math.log10(101),
        "1s": math.log10(1001),
        "16min": math.log10(960_001),
        "11days": math.log10(950_400_001),
    }

    fig.update_coloraxes(
        colorbar=dict(
            title="Clock Offset",
            tickvals=list(tick_map.values()),
            ticktext=list(tick_map.keys()),
        )
    )

    fig.update_traces(
        marker=dict(size=8, line=dict(width=0.5, color="black")),
    )

    fig.update_layout(
        title="ICMP Clock Synchronization Quality",
        geo=dict(
            showframe=False,
            showcoastlines=True,
            coastlinecolor="gray",
            showland=True,
            landcolor="rgb(243, 243, 243)",
        ),
        margin=dict(l=0, r=0, t=40, b=0),
    )

    return fig


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Plot ICMP clock sync data on a world map"
    )
    parser.add_argument("input", help="Path to enriched CSV file")
    parser.add_argument(
        "-o",
        "--output",
        default="clock_sync_map.html",
        help="Output file path (default: clock_sync_map.html)",
    )
    parser.add_argument(
        "--png",
        action="store_true",
        help="Also export a static PNG image",
    )
    args = parser.parse_args()

    df = load_and_filter(args.input)
    df = compute_log_offset(df)
    fig = build_figure(df)

    output = Path(args.output)

    if output.suffix == ".html":
        fig.write_html(str(output))
        print(f"Wrote interactive map to {output}")
    elif output.suffix in (".png", ".svg", ".pdf"):
        fig.write_image(str(output), width=1600, height=900, scale=2)
        print(f"Wrote static image to {output}")
    else:
        fig.write_html(str(output))
        print(f"Wrote {output}")

    if args.png and output.suffix == ".html":
        png_path = output.with_suffix(".png")
        fig.write_image(str(png_path), width=1600, height=900, scale=2)
        print(f"Wrote static image to {png_path}")


if __name__ == "__main__":
    main()
