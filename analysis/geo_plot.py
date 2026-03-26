import argparse
import math
from pathlib import Path

import pandas as pd
import plotly.express as px
import plotly.graph_objects as go


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


def load_data(csv_path: str) -> pd.DataFrame:
    df = pd.read_csv(csv_path)
    df = df[(df["rtt_ms"] > 0) & (df["rtt_ms"] <= 10000)]
    df = df.dropna(subset=["latitude", "longitude"])
    df["abs_offset_ms"] = df["clock_offset_ms"].abs()
    df["log_offset"] = df["abs_offset_ms"].apply(lambda x: math.log10(x + 1))
    df["offset_label"] = df["clock_offset_ms"].apply(fmt_offset)
    df["hostname_display"] = df["hostname"].fillna("(none)")
    return df


def _log_offset(ms: float) -> float:
    return math.log10(abs(ms) + 0.1)


def _td_ms(td: pd.Timedelta) -> float:
    return td.total_seconds() * 1000


# Human-readable colorbar ticks. This is what shows up in the key, so we want to
# state it in useful units (not giant ms)
_TICK_MS = {
    "0ms":  0,
    "10ms": _td_ms(pd.Timedelta("10ms")),
    "100ms": _td_ms(pd.Timedelta("100ms")),
    "1s":   _td_ms(pd.Timedelta("1s")),
    "1min": _td_ms(pd.Timedelta("1min")),
    "1hr":  _td_ms(pd.Timedelta("1hr")),
    "1day": _td_ms(pd.Timedelta("1day")),
}
TICK_MAP = {label: _log_offset(ms) for label, ms in _TICK_MS.items()}


def build_figure(df: pd.DataFrame) -> go.Figure:
    fig = px.scatter_geo(
        df,
        lat="latitude",
        lon="longitude",
        color="log_offset",
        color_continuous_scale="RdYlGn_r", # magic that handles colors idk
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
            "is_http": True,
            "had_date": True,
        },
        labels={
            "offset_label": "Clock Offset",
            "rtt_ms": "RTT (ms)",
            "ip": "IP",
            "hostname_display": "Hostname",
            "is_http": "Has HTTP",
            "had_date": "Has Date",
        },
    )

    fig.update_coloraxes(
        colorbar=dict(
            title="Clock Offset",
            tickvals=list(TICK_MAP.values()),
            ticktext=list(TICK_MAP.keys()),
        )
    )

    fig.update_traces(marker=dict(size=8, line=dict(width=0.5, color="black")))

    http_df = df.query("is_http == True")
    if not http_df.empty:
        fig.add_trace(
            go.Scattergeo(
                lat=http_df["latitude"],
                lon=http_df["longitude"],
                mode="markers",
                marker=dict(
                    size=14,
                    color="rgba(0,0,0,0)",
                    line=dict(width=2, color="black"),
                ),
                hoverinfo="skip",
                showlegend=True,
                name="Has HTTP",
            )
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


def write_figure(fig: go.Figure, path: Path, png: bool = False) -> None:
    suffix = path.suffix.lower()
    if suffix in (".png", ".svg", ".pdf"):
        fig.write_image(str(path), width=1600, height=900, scale=2)
        print(f"Wrote static image to {path}")
    else:
        fig.write_html(str(path))
        print(f"Wrote interactive map to {path}")
        if png:
            png_path = path.with_suffix(".png")
            fig.write_image(str(png_path), width=1600, height=900, scale=2)
            print(f"Wrote static image to {png_path}")


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
        help="Also export a static PNG image (only applies when output is HTML)",
    )
    args = parser.parse_args()

    df = load_data(args.input)
    fig = build_figure(df)
    write_figure(fig, Path(args.output), png=args.png)


if __name__ == "__main__":
    main()
