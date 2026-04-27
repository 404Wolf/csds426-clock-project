#!/usr/bin/env bash
set -euo pipefail

FIGURES_DIR="$(dirname "$0")/../report/figures"

fd -e mmd . "$FIGURES_DIR" --exec mmdc -i {} -o {.}.svg
