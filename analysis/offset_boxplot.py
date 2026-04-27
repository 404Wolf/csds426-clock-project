from pathlib import Path

import pandas as pd
import plotly.graph_objects as go
from plotly.subplots import make_subplots

ROOT = Path(__file__).parent.parent
OUTPUT = Path(__file__).parent / "out/offset_boxplot.html"

COLORS = {"ICMP": "steelblue", "HTTP (ICMP hosts)": "crimson", "HTTP (Tranco)": "darkorange"}


def iqr_filter(s: pd.Series) -> pd.Series:
    q1, q3 = s.quantile(0.25), s.quantile(0.75)
    iqr = q3 - q1
    return s[s.between(q1 - 1.5 * iqr, q3 + 1.5 * iqr)]


df = pd.read_csv(
    ROOT / "data/icmp_with_http.csv",
    usecols=["icmp_clock_offset_ms", "http_clock_offset_ms"],
)
df = df[df["icmp_clock_offset_ms"].notna() & df["http_clock_offset_ms"].notna()]
icmp = iqr_filter(df["icmp_clock_offset_ms"])
http = iqr_filter(df["http_clock_offset_ms"])

dt = pd.read_csv(ROOT / "data/tranco_http.csv", usecols=["http_clock_offset_ms"])
dt = dt[dt["http_clock_offset_ms"].notna()]
tranco = iqr_filter(dt["http_clock_offset_ms"])

series = {"ICMP": icmp, "HTTP (ICMP hosts)": http, "HTTP (Tranco)": tranco}

print("After IQR outlier removal:")
for label, s in series.items():
    print(
        f"  {label:18s}: n={len(s):,}  median={s.median():+.1f} ms  "
        f"std={s.std():.1f} ms  p90={s.abs().quantile(0.9):.1f} ms"
    )

fig = make_subplots(
    rows=1, cols=2,
    column_widths=[0.6, 0.4],
    subplot_titles=("ICMP & Tranco Top Sites", "HTTP (ICMP Hosts)"),
    shared_yaxes=False,
)

def box_trace(label, s):
    p90 = s.abs().quantile(0.9)
    return go.Box(
        y=s,
        name=f"{label}<br><sub>p90={p90:.0f} ms</sub>",
        marker_color=COLORS[label],
        boxmean="sd",
        line_width=1.5,
        marker=dict(size=3, opacity=0.4),
        boxpoints="outliers",
        jitter=0.3,
        whiskerwidth=0.6,
    )

fig.add_trace(box_trace("ICMP", icmp), row=1, col=1)
fig.add_trace(box_trace("HTTP (Tranco)", tranco), row=1, col=1)
fig.add_trace(box_trace("HTTP (ICMP hosts)", http), row=1, col=2)

axis_style = dict(
    zeroline=True,
    zerolinewidth=1.5,
    zerolinecolor="rgba(0,0,0,0.3)",
    gridcolor="rgba(0,0,0,0.08)",
)

fig.update_layout(
    title=dict(
        text="Internet Clock Desync: ICMP vs HTTP",
        x=0.5,
        font=dict(size=20),
    ),
    yaxis=dict(**axis_style, title="Clock Offset (ms)"),
    yaxis2=dict(**axis_style, title="Clock Offset (ms)"),
    plot_bgcolor="white",
    paper_bgcolor="white",
    showlegend=False,
    width=960,
    height=600,
    margin=dict(l=70, r=40, t=80, b=60),
)

OUTPUT.parent.mkdir(parents=True, exist_ok=True)
fig.write_html(str(OUTPUT))
print(f"Wrote {OUTPUT}")
