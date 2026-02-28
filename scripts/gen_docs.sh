#!/usr/bin/env bash
# Regenerate all SVG assets used in the visus documentation.
# Run from the repository root:
#   bash scripts/gen_docs.sh

set -euo pipefail

EXAMPLES=(
    band
    bar
    boxplot
    brick
    candlestick
    chord
    contour
    dotplot
    heatmap
    histogram
    histogram2d
    layout
    line
    manhattan
    phylo
    pie
    sankey
    scatter
    series
    stacked_area
    strip
    synteny
    upset
    violin
    volcano
    waterfall
    all_plots_simple
    all_plots_complex
)

echo "Building examples..."
cargo build --examples --quiet

echo "Generating doc SVGs..."
for ex in "${EXAMPLES[@]}"; do
    echo "  $ex"
    cargo run --example "$ex" --quiet
done

echo "Done."
