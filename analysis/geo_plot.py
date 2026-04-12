import argparse
import math
from pathlib import Path

import pandas as pd
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


def _enrich(df: pd.DataFrame) -> pd.DataFrame:
    df = df[(df["rtt_ms"] > 0) & (df["rtt_ms"] <= 10000)]
    df = df.dropna(subset=["latitude", "longitude"])
    df["log_offset"] = df["clock_offset_ms"].abs().apply(lambda x: math.log10(x + 1))
    df["offset_label"] = df["clock_offset_ms"].apply(fmt_offset)
    df["hostname_display"] = df["hostname"].fillna("(none)")
    return df


def load_new_data(csv_path: str) -> pd.DataFrame:
    df = pd.read_csv(csv_path)
    df = df.rename(columns={"icmp_rtt_ms": "rtt_ms", "icmp_clock_offset_ms": "clock_offset_ms"})
    df["is_http"] = df["http_clock_offset_ms"].notna()
    return _enrich(df)



def load_clockdiff_data(csv_path: str) -> pd.DataFrame:
    df = pd.read_csv(csv_path)
    df["is_http"] = df["is_http"].astype(str).str.lower() == "true"
    return _enrich(df)


# Human-readable colorbar ticks
_TICK_MAP = {
    "0ms":   math.log10(0 + 1),
    "10ms":  math.log10(10 + 1),
    "100ms": math.log10(100 + 1),
    "1s":    math.log10(1_000 + 1),
    "1min":  math.log10(60_000 + 1),
    "1hr":   math.log10(3_600_000 + 1),
    "1day":  math.log10(86_400_000 + 1),
}


def build_figure(df: pd.DataFrame, clockdiff_df: pd.DataFrame | None = None) -> go.Figure:
    hover = df.apply(
        lambda r: (
            f"Offset: {r['offset_label']}<br>"
            f"RTT: {r['rtt_ms']}ms<br>"
            f"IP: {r['ip']}<br>"
            f"Host: {r['hostname_display']}<br>"
            f"Location: {r['city']}, {r['country']}<br>"
            f"Has HTTP: {r['is_http']}"
        ),
        axis=1,
    )

    traces = [
        go.Scattergeo(
            lat=df["latitude"],
            lon=df["longitude"],
            mode="markers",
            marker=dict(
                size=3,
                color=df["log_offset"],
                colorscale="RdYlGn_r",
                cmin=0,
                cmax=df["log_offset"].quantile(0.95),
                colorbar=dict(
                    title="Clock Offset",
                    tickvals=list(_TICK_MAP.values()),
                    ticktext=list(_TICK_MAP.keys()),
                ),
                line=dict(width=0.5, color="black"),
            ),
            text=hover,
            hovertemplate="%{text}<extra></extra>",
            showlegend=False,
        )
    ]

    if clockdiff_df is not None:
        http_df = clockdiff_df[clockdiff_df["is_http"]]
        if not http_df.empty:
            traces.append(go.Scattergeo(
                lat=http_df["latitude"],
                lon=http_df["longitude"],
                mode="markers",
                marker=dict(
                    size=500,
                    color="rgba(0,0,0,0)",
                    line=dict(width=0.002, color="black"),
                ),
                hoverinfo="skip",
                showlegend=True,
                name="Has HTTP",
            ))

    return go.Figure(
        data=traces,
        layout=go.Layout(
            title="ICMP Clock Synchronization Quality",
            geo=dict(
                projection_type="natural earth",
                showframe=False,
                showcoastlines=True,
                coastlinecolor="gray",
                showland=True,
                landcolor="rgb(243, 243, 243)",
            ),
            margin=dict(l=0, r=0, t=40, b=0),
        ),
    )


def main() -> None:
    parser = argparse.ArgumentParser(description="Plot ICMP clock sync data on a world map")
    parser.add_argument("input", help="Path to enriched CSV file (new format)")
    parser.add_argument("clockdiff", help="Path to clockdiff CSV (old format); black circles mark is_http rows")
    parser.add_argument("-o", "--output", default="clock_sync_map")
    parser.add_argument("--svg", action="store_true", help="Also save an SVG")
    args = parser.parse_args()

    new_df = load_new_data(args.input)
    clockdiff_df = load_clockdiff_data(args.clockdiff)
    df = pd.concat([new_df, clockdiff_df], ignore_index=True)

    fig = build_figure(df, clockdiff_df)
    html_out = Path(args.output).with_suffix(".html")
    fig.write_html(str(html_out))
    print(f"Wrote {html_out}")

    if args.svg:
        svg_out = Path(args.output).with_suffix(".svg")
        fig.write_image(str(svg_out))
        print(f"Wrote {svg_out}")


if __name__ == "__main__":
    main()
