import argparse
import math
from pathlib import Path

import cartopy.crs as ccrs
import cartopy.feature as cfeature
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
from matplotlib.colors import LogNorm
from scipy.interpolate import griddata


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


# Colorbar tick positions and labels (log10 scale)
_TICK_VALS = [math.log10(v + 1) for v in [0, 10, 100, 1_000, 60_000, 3_600_000, 86_400_000]]
_TICK_LABELS = ["0ms", "10ms", "100ms", "1s", "1min", "1hr", "1day"]

_GRID_RES = 200  # interpolation grid resolution (lon × lat)


def build_figure(df: pd.DataFrame, clockdiff_df: pd.DataFrame | None = None) -> plt.Figure:
    lon_grid, lat_grid = np.meshgrid(
        np.linspace(-180, 180, _GRID_RES * 2),
        np.linspace(-90, 90, _GRID_RES),
    )
    z = griddata(
        (df["longitude"].values, df["latitude"].values),
        df["log_offset"].values,
        (lon_grid, lat_grid),
        method="linear",
    )

    vmax = float(df["log_offset"].quantile(0.95))

    fig = plt.figure(figsize=(14, 7))
    ax = fig.add_subplot(1, 1, 1, projection=ccrs.EqualEarth())
    ax.set_global()
    ax.add_feature(cfeature.LAND, facecolor="#f3f3f3")
    ax.add_feature(cfeature.COASTLINE, linewidth=0.5, edgecolor="gray")
    ax.add_feature(cfeature.BORDERS, linewidth=0.3, edgecolor="lightgray")

    mesh = ax.pcolormesh(
        lon_grid, lat_grid, z,
        cmap="RdYlGn_r",
        vmin=0, vmax=vmax,
        transform=ccrs.PlateCarree(),
        shading="gouraud",
    )

    cbar = fig.colorbar(mesh, ax=ax, orientation="vertical", fraction=0.03, pad=0.02)
    cbar.set_label("Clock Offset")
    cbar.set_ticks(_TICK_VALS)
    cbar.set_ticklabels(_TICK_LABELS)

    ax.set_title("ICMP Clock Synchronization Quality")
    return fig


def main() -> None:
    parser = argparse.ArgumentParser(description="Plot ICMP clock sync data on a world map")
    parser.add_argument("input", help="Path to enriched CSV file (new format)")
    parser.add_argument("clockdiff", help="Path to clockdiff CSV (old format); black circles mark is_http rows")
    parser.add_argument("-o", "--output", default="out/clock_sync_map")
    parser.add_argument("--svg", action="store_true", help="Also save an SVG")
    args = parser.parse_args()

    new_df = load_new_data(args.input)
    clockdiff_df = load_clockdiff_data(args.clockdiff)
    df = pd.concat([new_df, clockdiff_df], ignore_index=True)

    fig = build_figure(df, clockdiff_df)
    png_out = Path(args.output).with_suffix(".png")
    fig.savefig(str(png_out), dpi=150, bbox_inches="tight")
    print(f"Wrote {png_out}")

    if args.svg:
        svg_out = Path(args.output).with_suffix(".svg")
        fig.savefig(str(svg_out), bbox_inches="tight")
        print(f"Wrote {svg_out}")


if __name__ == "__main__":
    main()
