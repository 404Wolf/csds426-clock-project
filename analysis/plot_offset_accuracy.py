#!/usr/bin/env python3
"""Box plots of measurement error by clock offset, split by host group."""

import argparse
import pandas as pd
import plotly.graph_objects as go
from plotly.subplots import make_subplots

ap = argparse.ArgumentParser()
ap.add_argument("csv", nargs="?", default="../tune_measurements.csv")
args = ap.parse_args()

df = pd.read_csv(args.csv)
trial_col = "trial" if "trial" in df.columns else "run"
df = df.groupby([trial_col, "host", "offset_s"], as_index=False)["err_us"].min()

far_host = "149.28.167.248:8080"
groups = [
    ("Far (149.x)", df[df["host"] == far_host]),
    ("Local (66.x)", df[df["host"] != far_host]),
]

offsets = sorted(df["offset_s"].unique())
fig = make_subplots(rows=1, cols=2, subplot_titles=[g[0] for g in groups])

for col, (title, gdf) in enumerate(groups, start=1):
    for off in offsets:
        vals = gdf[gdf["offset_s"] == off]["err_us"] / 1000
        fig.add_trace(go.Box(y=vals, name=str(off), boxpoints=False, showlegend=(col == 1)),
                      row=1, col=col)

    for p in [0.50, 0.90]:
        val = gdf["err_us"].quantile(p) / 1000
        fig.add_hline(y=val, line_dash="dash", line_color="red", row=1, col=col)
        fig.add_annotation(
            x=1.0, xref=f"x{col if col > 1 else ''} domain",
            y=val, yref=f"y{col if col > 1 else ''}",
            text=f"p{int(p*100)} = {val:.1f}ms",
            showarrow=False, yanchor="bottom", xanchor="right",
        )
        print(f"{title} p{int(p*100)}: {val:.1f}ms")

p99_ms = df["err_us"].quantile(0.99) / 1000
fig.update_yaxes(title_text="|error| (ms)", col=1)
fig.update_yaxes(range=[0, p99_ms])
fig.update_xaxes(title_text="Clock offset (s)")
fig.update_layout(title="Measurement error by clock offset")
fig.write_html("plot_offset_accuracy.html")
print("saved plot_offset_accuracy.html")

fig2 = go.Figure()
for i, (title, gdf) in enumerate(groups):
    vals = gdf["err_us"] / 1000
    p90 = vals.quantile(0.90)
    fig2.add_trace(go.Box(y=vals, name=title, boxpoints=False))
    fig2.add_annotation(
        x=i, y=p90,
        text=f"p90={p90:.1f}ms",
        showarrow=True, arrowhead=2, arrowsize=0.8,
        ay=-30, ax=40,
        font=dict(size=12),
    )

fig2.update_yaxes(title_text="|error| (ms)", range=[0, p99_ms])
fig2.update_layout(title="Measurement error: Far vs Near (all offsets)")
fig2.write_html("plot_offset_accuracy_aggregate.html")
print("saved plot_offset_accuracy_aggregate.html")
