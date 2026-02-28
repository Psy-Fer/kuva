#!/usr/bin/env bash
# Render all CLI plot types to the terminal for visual inspection.
#
# Usage:
#   bash scripts/terminal_plots.sh                              # debug build, auto-detect size
#   bash scripts/terminal_plots.sh ./target/release/kuva       # release build, auto-detect size
#   bash scripts/terminal_plots.sh - 120 40                     # debug build, fixed 120×40
#   bash scripts/terminal_plots.sh ./target/release/kuva 80 24 # release build, fixed 80×24
#
# Pass '-' as the binary to use cargo run.
# Width and height default to the current terminal size when omitted.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DATA="$REPO_ROOT/examples/data"

# ── Binary selection ──────────────────────────────────────────────────────────
BIN="${1:--}"
if [[ "$BIN" == "-" ]]; then
    cd "$REPO_ROOT"
    run() { cargo run --quiet --bin kuva --features cli -- "$@"; }
else
    run() { "$BIN" "$@"; }
fi

# ── Terminal dimensions ───────────────────────────────────────────────────────
COLS="${2:-$(tput cols  2>/dev/null || echo 100)}"
ROWS="${3:-$(tput lines 2>/dev/null || echo 30)}"
PLOT_ROWS=$(( ROWS > 8 ? ROWS - 5 : 10 ))

W="--term-width $COLS"
H="--term-height $PLOT_ROWS"

header() {
    printf '\n\033[1;36m══ %s ══\033[0m\n' "$1"
}

# ── scatter ───────────────────────────────────────────────────────────────────
header "scatter"
run scatter "$DATA/scatter.tsv" --x x --y y --color-by group \
    --title "Scatter Plot" --x-label "X" --y-label "Y" \
    --terminal $W $H

# ── line ─────────────────────────────────────────────────────────────────────
header "line"
run line "$DATA/measurements.tsv" --x time --y value --color-by group \
    --title "Growth Curves" --x-label "Time" --y-label "Value" \
    --terminal $W $H

# ── bar ──────────────────────────────────────────────────────────────────────
header "bar"
run bar "$DATA/bar.tsv" --label-col category --value-col count \
    --title "Category Counts" --x-label "Category" --y-label "Count" \
    --terminal $W $H

# ── histogram ─────────────────────────────────────────────────────────────────
header "histogram"
run histogram "$DATA/histogram.tsv" --value-col value \
    --title "Value Distribution" --x-label "Value" --y-label "Count" \
    --terminal $W $H

# ── boxplot ───────────────────────────────────────────────────────────────────
header "boxplot"
run box "$DATA/samples.tsv" --group-col group --value-col expression \
    --title "Expression by Group" --x-label "Group" --y-label "Expression" \
    --terminal $W $H

# ── violin ───────────────────────────────────────────────────────────────────
header "violin"
run violin "$DATA/samples.tsv" --group-col group --value-col expression \
    --title "Expression Distribution" --x-label "Group" --y-label "Expression" \
    --terminal $W $H

# ── strip ────────────────────────────────────────────────────────────────────
header "strip"
run strip "$DATA/samples.tsv" --group-col group --value-col expression \
    --title "Expression Spread" --x-label "Group" --y-label "Expression" \
    --terminal $W $H

# ── pie ──────────────────────────────────────────────────────────────────────
header "pie"
run pie "$DATA/pie.tsv" --label-col feature --value-col percentage \
    --title "Genome Composition" \
    --terminal $W $H

# ── waterfall ─────────────────────────────────────────────────────────────────
header "waterfall"
run waterfall "$DATA/waterfall.tsv" --label-col process --value-col log2fc \
    --title "Log2 Fold Change" --x-label "Process" --y-label "log2FC" \
    --terminal $W $H

# ── stacked-area ─────────────────────────────────────────────────────────────
header "stacked-area"
run stacked-area "$DATA/stacked_area.tsv" --x-col week --group-col species --y-col abundance \
    --title "Species Abundance" --x-label "Week" --y-label "Abundance" \
    --terminal $W $H

# ── volcano ───────────────────────────────────────────────────────────────────
header "volcano"
run volcano "$DATA/volcano.tsv" --name-col gene --x-col log2fc --y-col pvalue \
    --title "Differential Expression" --x-label "log2 Fold Change" "--y-label=-log10(p-value)" \
    --terminal $W $H

# ── manhattan ─────────────────────────────────────────────────────────────────
header "manhattan"
run manhattan "$DATA/gene_stats.tsv" --chr-col chr --pvalue-col pvalue \
    --title "GWAS Results" --x-label "Chromosome" "--y-label=-log10(p-value)" \
    --terminal $W $H

# ── candlestick ───────────────────────────────────────────────────────────────
header "candlestick"
run candlestick "$DATA/candlestick.tsv" \
    --label-col date --open-col open --high-col high --low-col low --close-col close \
    --title "Stock Price" --x-label "Date" --y-label "Price (USD)" \
    --terminal $W $H

# ── heatmap ───────────────────────────────────────────────────────────────────
header "heatmap"
run heatmap "$DATA/heatmap.tsv" \
    --title "Gene Expression Heatmap" --x-label "Sample" --y-label "Gene" \
    --terminal $W $H

# ── hist2d ───────────────────────────────────────────────────────────────────
header "hist2d"
run hist2d "$DATA/hist2d.tsv" --x x --y y \
    --title "2D Density" --x-label "X" --y-label "Y" \
    --terminal $W $H

# ── contour ───────────────────────────────────────────────────────────────────
header "contour"
run contour "$DATA/contour.tsv" --x x --y y --z density \
    --title "Density Contour" --x-label "X" --y-label "Y" \
    --terminal $W $H

# ── dot ───────────────────────────────────────────────────────────────────────
header "dot"
run dot "$DATA/dot.tsv" --x-col pathway --y-col cell_type \
    --size-col pct_expressed --color-col mean_expr \
    --title "Gene Expression by Cell Type" --x-label "Pathway" --y-label "Cell Type" \
    --terminal $W $H

# ── upset ─────────────────────────────────────────────────────────────────────
header "upset"
run upset "$DATA/upset.tsv" --title "Set Intersections" \
    --terminal $W $H

# ── chord ─────────────────────────────────────────────────────────────────────
header "chord"
run chord "$DATA/chord.tsv" --title "Cell Type Co-occurrence" \
    --terminal $W $H

# ── sankey ───────────────────────────────────────────────────────────────────
header "sankey"
run sankey "$DATA/sankey.tsv" --source-col source --target-col target --value-col value \
    --title "Flow Diagram" \
    --terminal $W $H

# ── phylo ────────────────────────────────────────────────────────────────────
header "phylo"
run phylo "$DATA/phylo.tsv" --parent-col parent --child-col child --length-col length \
    --title "Phylogenetic Tree" --branch-color white \
    --terminal $W $H

# ── synteny ───────────────────────────────────────────────────────────────────
header "synteny"
run synteny "$DATA/synteny_seqs.tsv" --blocks-file "$DATA/synteny_blocks.tsv" \
    --title "Synteny Map" \
    --terminal $W $H

echo
