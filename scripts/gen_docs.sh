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
    datetime
    density
    diceplot
    dotplot
    ecdf
    forest
    funnel
    gantt
    heatmap
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
    math
    mosaic
    network
    parallel
    phylo
    pie
    polar
    pr
    pyramid
    qq
    quiver
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
    text
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

# ── Benchmark charts ───────────────────────────────────────────────────────────
# If a benchmark_results.csv exists, rebuild plot_results and regenerate the
# comparison SVGs, then copy them into docs/src/assets/bench/.
BENCH_DIR="benches/rust_bench"
BENCH_CSV="$BENCH_DIR/benchmark_results.csv"
BENCH_ASSET_DIR="docs/src/assets/bench"

if [[ -f "$BENCH_CSV" ]]; then
    echo "Regenerating benchmark charts from $BENCH_CSV..."
    (cd "$BENCH_DIR" && cargo build --release --bin plot_results --quiet && ./target/release/plot_results)
    mkdir -p "$BENCH_ASSET_DIR"
    cp "$BENCH_DIR"/output/bench_*.svg "$BENCH_DIR"/output/kuva_delta_*.svg "$BENCH_ASSET_DIR/"
    echo "Copied benchmark SVGs → $BENCH_ASSET_DIR/"
else
    echo "Skipping benchmark charts ($BENCH_CSV not found; run benches/rust_bench/run_benchmarks.sh first)."
fi
