#!/usr/bin/env python3
"""Box plots of measurement error by clock offset."""

import pandas as pd
import plotly.graph_objects as go

df = pd.read_csv("../tune_measurements.csv")
df = df.groupby(["trial", "host", "offset_s"], as_index=False)["err_us"].min()

offsets = sorted(df["offset_s"].unique())
fig = go.Figure()
for off in offsets:
    vals = df[df["offset_s"] == off]["err_us"] / 1000
    fig.add_trace(go.Box(y=vals, name=str(off), boxpoints=False))

fig.update_layout(
    title="Measurement error distribution by clock offset",
    xaxis_title="Clock offset (s)",
    yaxis_title="|error| (ms)",
    showlegend=False,
)
for p, xanchor in [(0.50, 0.99), (0.90, 0.99), (0.95, 0.99), (0.99, 0.99)]:
    val = df["err_us"].quantile(p) / 1000
    fig.add_hline(y=val, line_dash="dash", line_color="red")
    fig.add_annotation(
        x=xanchor, xref="paper",
        y=val, yref="y",
        text=f"p{int(p*100)} = {val:.1f}ms",
        showarrow=False, yanchor="bottom",
    )
    print(f"p{int(p*100)} error: {val:.1f}ms")
fig.write_html("plot_offset_accuracy.html")
print("saved plot_offset_accuracy.html")
