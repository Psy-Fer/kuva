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
    bump
    calendar
    candlestick
    chord
    clustermap
    contour
    density
    diceplot
    ecdf
    forest
    funnel
    gantt
    hexbin
    histogram
    histogram2d
    horizon
    jointplot
    layout
    legend_plot
    legends
    line
    lollipop
    manhattan
    mosaic
    network
    parallel
    phylo
    pie
    polar
    pr
    pyramid
    qq
    radar
    raincloud
    ridgeline
    roc
    rose
    sankey
    scale
    scatter
    scatter3d
    series
    slope
    stacked_area
    streamgraph
    strip
    sunburst
    surface3d
    survival
    synteny
    ternary
    treemap
    twin_y
    upset
    venn
    violin
    volcano
    waffle
    waterfall
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
