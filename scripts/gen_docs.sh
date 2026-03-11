#!/usr/bin/env bash
# Regenerate all SVG assets used in the kuva documentation.
# Run from the repository root:
#   bash scripts/gen_docs.sh

set -euo pipefail

EXAMPLES=(
    band
    bar
    figure
    boxplot
    brick
    candlestick
    chord
    contour
    density
    ridgeline
    dotplot
    heatmap
    histogram
    histogram2d
    layout
    legends
    line
    manhattan
    phylo
    pie
    sankey
    scale
    scatter
    series
    stacked_area
    strip
    synteny
    upset
    violin
    volcano
    waterfall
    polar
    ternary
    all_plots_simple
    all_plots_complex
)

echo "Building examples..."
cargo build --features full --examples --quiet

echo "Generating doc SVGs..."
for ex in "${EXAMPLES[@]}"; do
    echo "  $ex"
    cargo run --features full --example "$ex" --quiet
done

echo "Done."
