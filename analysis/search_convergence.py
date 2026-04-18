import argparse
from pathlib import Path

import numpy as np
import pandas as pd
import plotly.graph_objects as go
from plotly.subplots import make_subplots


def main() -> None:
    parser = argparse.ArgumentParser(description="Visualise binary search convergence across rounds")
    parser.add_argument("input", help="enrich-http output CSV")
    parser.add_argument("-o", "--output", default="out/search_convergence.html")
    parser.add_argument("--svg", action="store_true")
    args = parser.parse_args()

    df = pd.read_csv(args.input)

    window_cols = sorted(
        [c for c in df.columns if c.startswith("round_") and c.endswith("_window_ms")],
        key=lambda c: int(c.split("_")[1]),
    )
    rounds = [int(c.split("_")[1]) for c in window_cols]

    # Per-round stats
    pct_reached = []   # % of hosts that reached this round (non-null)
    medians, p25s, p75s, p10s, p90s = [], [], [], [], []
    thresholds = [500, 100, 10, 1]   # ms
    pct_within = {t: [] for t in thresholds}

    total = len(df)
    for col in window_cols:
        s = df[col].dropna()
        pct_reached.append(100 * len(s) / total)
        medians.append(s.median() if len(s) else np.nan)
        p25s.append(s.quantile(0.25) if len(s) else np.nan)
        p75s.append(s.quantile(0.75) if len(s) else np.nan)
        p10s.append(s.quantile(0.10) if len(s) else np.nan)
        p90s.append(s.quantile(0.90) if len(s) else np.nan)
        for t in thresholds:
            pct_within[t].append(100 * (s <= t).sum() / total)

    fig = make_subplots(
        rows=1, cols=2,
        subplot_titles=(
            "Window size by round (hosts that reached each round)",
            "Cumulative % of hosts within threshold by round",
        ),
    )

    # Left: window distribution per round
    fig.add_trace(go.Scatter(
        x=rounds, y=p90s, mode="lines", line=dict(width=0),
        showlegend=False, hoverinfo="skip",
    ), row=1, col=1)
    fig.add_trace(go.Scatter(
        x=rounds, y=p10s, mode="lines", fill="tonexty",
        fillcolor="rgba(70,130,180,0.15)", line=dict(width=0),
        name="p10–p90", showlegend=True,
    ), row=1, col=1)
    fig.add_trace(go.Scatter(
        x=rounds, y=p75s, mode="lines", line=dict(width=0),
        showlegend=False, hoverinfo="skip",
    ), row=1, col=1)
    fig.add_trace(go.Scatter(
        x=rounds, y=p25s, mode="lines", fill="tonexty",
        fillcolor="rgba(70,130,180,0.3)", line=dict(width=0),
        name="p25–p75", showlegend=True,
    ), row=1, col=1)
    fig.add_trace(go.Scatter(
        x=rounds, y=medians, mode="lines+markers",
        line=dict(color="steelblue", width=2),
        marker=dict(size=5),
        name="median window",
        customdata=list(zip(pct_reached, medians)),
        hovertemplate="Round %{x}<br>Median window: %{y:.0f}ms<br>Hosts reached: %{customdata[0]:.1f}%<extra></extra>",
    ), row=1, col=1)

    # Right: % within threshold per round
    colors = ["crimson", "darkorange", "mediumseagreen", "steelblue"]
    for t, color in zip(thresholds, colors):
        fig.add_trace(go.Scatter(
            x=rounds, y=pct_within[t], mode="lines+markers",
            line=dict(color=color, width=2), marker=dict(size=5),
            name=f"≤{t}ms",
        ), row=1, col=2)

    fig.update_xaxes(title_text="Round", dtick=1, row=1, col=1)
    fig.update_xaxes(title_text="Round", dtick=1, row=1, col=2)
    fig.update_yaxes(title_text="Window half-span (ms)", type="log", row=1, col=1)
    fig.update_yaxes(title_text="% of all hosts", row=1, col=2)

    fig.update_layout(
        title="Binary Search Convergence",
        margin=dict(l=60, r=20, t=60, b=60),
        width=1200, height=500,
        legend=dict(x=0.55, y=0.95),
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


if __name__ == "__main__":
    main()
