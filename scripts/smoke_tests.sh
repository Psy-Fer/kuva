#!/usr/bin/env bash
# Smoke tests for the visus CLI binary.
# Runs every subcommand against example data and checks that SVG output is produced.
#
# Usage:
#   ./scripts/smoke_tests.sh [path/to/visus] [--save [dir]]
#
# Options:
#   First non-flag arg   Path to visus binary (default: ./target/debug/visus)
#   --save [dir]         Write each SVG to a file in dir (default: smoke_test_outputs/)
#                        so you can visually inspect results in a browser.

set -euo pipefail

BIN=""
SAVE=0
OUTDIR="smoke_test_outputs"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --save)
            SAVE=1
            # Optional directory argument after --save
            if [[ $# -gt 1 && "$2" != --* ]]; then
                OUTDIR="$2"
                shift
            fi
            shift
            ;;
        *)
            BIN="$1"
            shift
            ;;
    esac
done

BIN="${BIN:-./target/debug/visus}"
DATA="./examples/data"

if [[ $SAVE -eq 1 ]]; then
    mkdir -p "$OUTDIR"
    echo "Saving SVG outputs to: $OUTDIR/"
    echo ""
fi

PASS=0
FAIL=0

check() {
    local name="$1"
    shift
    local fname
    fname="${name// /_}"

    if [[ $SAVE -eq 1 ]]; then
        local outfile="$OUTDIR/${fname}.svg"
        if "$@" | tee "$outfile" | grep -q "<svg"; then
            echo "PASS  $name  →  $outfile"
            PASS=$((PASS + 1))
        else
            echo "FAIL  $name"
            FAIL=$((FAIL + 1))
        fi
    else
        if "$@" | grep -q "<svg"; then
            echo "PASS  $name"
            PASS=$((PASS + 1))
        else
            echo "FAIL  $name"
            FAIL=$((FAIL + 1))
        fi
    fi
}

# ── scatter ───────────────────────────────────────────────────────────────────
check "scatter basic" \
    "$BIN" scatter "$DATA/scatter.tsv" --x x --y y \
        --title "Scatter Plot" --x-label "X" --y-label "Y"

check "scatter color-by" \
    "$BIN" scatter "$DATA/scatter.tsv" --x x --y y --color-by group --legend \
        --title "Scatter by Group" --x-label "X" --y-label "Y"

check "scatter trend" \
    "$BIN" scatter "$DATA/scatter.tsv" --x x --y y --trend --equation --correlation \
        --title "Scatter with Trend" --x-label "X" --y-label "Y"

# ── line ──────────────────────────────────────────────────────────────────────
check "line color-by" \
    "$BIN" line "$DATA/measurements.tsv" --x time --y value --color-by group \
        --title "Growth Curves" --x-label "Time" --y-label "Value"

check "line color-by legend" \
    "$BIN" line "$DATA/measurements.tsv" --x time --y value --color-by group --legend \
        --title "Growth Curves" --x-label "Time" --y-label "Value"

# ── bar ───────────────────────────────────────────────────────────────────────
check "bar basic" \
    "$BIN" bar "$DATA/bar.tsv" --label-col category --value-col count \
        --title "Category Counts" --x-label "Category" --y-label "Count"

# ── histogram ─────────────────────────────────────────────────────────────────
check "histogram basic" \
    "$BIN" histogram "$DATA/histogram.tsv" --value-col value \
        --title "Value Distribution" --x-label "Value" --y-label "Count"

check "histogram bins" \
    "$BIN" histogram "$DATA/histogram.tsv" --value-col value --bins 20 \
        --title "Value Distribution" --x-label "Value" --y-label "Count"

check "histogram normalize" \
    "$BIN" histogram "$DATA/histogram.tsv" --value-col value --normalize \
        --title "Value Distribution" --x-label "Value" --y-label "Density"

# ── box ───────────────────────────────────────────────────────────────────────
check "box basic" \
    "$BIN" box "$DATA/samples.tsv" --group-col group --value-col expression \
        --title "Expression by Group" --x-label "Group" --y-label "Expression"

check "box overlay-points" \
    "$BIN" box "$DATA/samples.tsv" --group-col group --value-col expression --overlay-points \
        --title "Expression by Group" --x-label "Group" --y-label "Expression"

check "box overlay-swarm" \
    "$BIN" box "$DATA/samples.tsv" --group-col group --value-col expression --overlay-swarm \
        --title "Expression by Group" --x-label "Group" --y-label "Expression"

# ── violin ────────────────────────────────────────────────────────────────────
check "violin basic" \
    "$BIN" violin "$DATA/samples.tsv" --group-col group --value-col expression \
        --title "Expression Distribution" --x-label "Group" --y-label "Expression"

check "violin overlay-swarm" \
    "$BIN" violin "$DATA/samples.tsv" --group-col group --value-col expression --overlay-swarm \
        --title "Expression Distribution" --x-label "Group" --y-label "Expression"

# ── pie ───────────────────────────────────────────────────────────────────────
check "pie basic" \
    "$BIN" pie "$DATA/pie.tsv" --label-col feature --value-col percentage \
        --title "Genome Composition"

check "pie donut percent" \
    "$BIN" pie "$DATA/pie.tsv" --label-col feature --value-col percentage --donut --percent --legend \
        --title "Genome Composition"

# ── strip ─────────────────────────────────────────────────────────────────────
check "strip jitter" \
    "$BIN" strip "$DATA/samples.tsv" --group-col group --value-col expression \
        --title "Expression Spread" --x-label "Group" --y-label "Expression"

check "strip swarm" \
    "$BIN" strip "$DATA/samples.tsv" --group-col group --value-col expression --swarm \
        --title "Expression Spread" --x-label "Group" --y-label "Expression"

check "strip center" \
    "$BIN" strip "$DATA/samples.tsv" --group-col group --value-col expression --center \
        --title "Expression Spread" --x-label "Group" --y-label "Expression"

# ── waterfall ─────────────────────────────────────────────────────────────────
check "waterfall basic" \
    "$BIN" waterfall "$DATA/waterfall.tsv" --label-col process --value-col log2fc \
        --title "Log2 Fold Change" --x-label "Process" --y-label "log2FC"

check "waterfall connectors values" \
    "$BIN" waterfall "$DATA/waterfall.tsv" --label-col process --value-col log2fc --connectors --values \
        --title "Log2 Fold Change" --x-label "Process" --y-label "log2FC"

# ── stacked-area ──────────────────────────────────────────────────────────────
check "stacked-area basic" \
    "$BIN" stacked-area "$DATA/stacked_area.tsv" --x-col week --group-col species --y-col abundance \
        --title "Species Abundance" --x-label "Week" --y-label "Abundance"

check "stacked-area normalize" \
    "$BIN" stacked-area "$DATA/stacked_area.tsv" --x-col week --group-col species --y-col abundance --normalize \
        --title "Relative Abundance" --x-label "Week" --y-label "Proportion"

# ── volcano ───────────────────────────────────────────────────────────────────
check "volcano basic" \
    "$BIN" volcano "$DATA/volcano.tsv" --name-col gene --x-col log2fc --y-col pvalue \
        --title "Differential Expression" --x-label "log2 Fold Change" "--y-label=-log10(p-value)"

check "volcano top-n legend" \
    "$BIN" volcano "$DATA/volcano.tsv" --name-col gene --x-col log2fc --y-col pvalue --top-n 10 --legend \
        --title "Differential Expression" --x-label "log2 Fold Change" "--y-label=-log10(p-value)"

# ── manhattan ─────────────────────────────────────────────────────────────────
check "manhattan sequential" \
    "$BIN" manhattan "$DATA/gene_stats.tsv" --chr-col chr --pvalue-col pvalue \
        --title "GWAS Results" --x-label "Chromosome" "--y-label=-log10(p-value)"

check "manhattan top-n" \
    "$BIN" manhattan "$DATA/gene_stats.tsv" --chr-col chr --pvalue-col pvalue --top-n 10 \
        --title "GWAS Results" --x-label "Chromosome" "--y-label=-log10(p-value)"

check "manhattan hg38" \
    "$BIN" manhattan "$DATA/gene_stats.tsv" --chr-col chr --pos-col pos --pvalue-col pvalue --genome-build hg38 \
        --title "GWAS Results (hg38)" --x-label "Chromosome" "--y-label=-log10(p-value)"

# ── candlestick ───────────────────────────────────────────────────────────────
check "candlestick basic" \
    "$BIN" candlestick "$DATA/candlestick.tsv" \
        --label-col date --open-col open --high-col high --low-col low --close-col close \
        --title "Stock Price" --x-label "Date" --y-label "Price (USD)"

check "candlestick volume panel" \
    "$BIN" candlestick "$DATA/candlestick.tsv" \
        --label-col date --open-col open --high-col high --low-col low --close-col close \
        --volume-col volume --volume-panel \
        --title "Stock Price with Volume" --x-label "Date" --y-label "Price (USD)"

# ── heatmap ───────────────────────────────────────────────────────────────────
check "heatmap basic" \
    "$BIN" heatmap "$DATA/heatmap.tsv" \
        --title "Gene Expression Heatmap" --x-label "Sample" --y-label "Gene"

check "heatmap values inferno" \
    "$BIN" heatmap "$DATA/heatmap.tsv" --values --colormap inferno --legend "z-score" --height 800 \
        --title "Gene Expression Heatmap" --x-label "Sample" --y-label "Gene"

# ── hist2d ────────────────────────────────────────────────────────────────────
check "hist2d basic" \
    "$BIN" hist2d "$DATA/hist2d.tsv" --x x --y y \
        --title "2D Density" --x-label "X" --y-label "Y"

check "hist2d fine bins" \
    "$BIN" hist2d "$DATA/hist2d.tsv" --x x --y y --bins-x 30 --bins-y 30 --correlation \
        --title "2D Density" --x-label "X" --y-label "Y"

# ── contour ───────────────────────────────────────────────────────────────────
check "contour basic" \
    "$BIN" contour "$DATA/contour.tsv" --x x --y y --z density \
        --title "Density Contour" --x-label "X" --y-label "Y"

check "contour filled" \
    "$BIN" contour "$DATA/contour.tsv" --x x --y y --z density --filled --levels 10 --legend "density" \
        --title "Density Contour" --x-label "X" --y-label "Y"

# ── dot ───────────────────────────────────────────────────────────────────────
check "dot basic" \
    "$BIN" dot "$DATA/dot.tsv" --x-col pathway --y-col cell_type \
        --size-col pct_expressed --color-col mean_expr \
        --title "Gene Expression by Cell Type" --x-label "Pathway" --y-label "Cell Type"

check "dot legend colorbar" \
    "$BIN" dot "$DATA/dot.tsv" --x-col pathway --y-col cell_type \
        --size-col pct_expressed --color-col mean_expr \
        --size-legend "% expressed" --colorbar "mean expr" \
        --title "Gene Expression by Cell Type" --x-label "Pathway" --y-label "Cell Type"

# ── upset ─────────────────────────────────────────────────────────────────────
check "upset basic" \
    "$BIN" upset "$DATA/upset.tsv" --title "Set Intersections"

check "upset sort degree" \
    "$BIN" upset "$DATA/upset.tsv" --sort degree --max-visible 10 --title "Set Intersections"

# ── chord ─────────────────────────────────────────────────────────────────────
check "chord basic" \
    "$BIN" chord "$DATA/chord.tsv" --title "Cell Type Co-occurrence"

check "chord gap legend" \
    "$BIN" chord "$DATA/chord.tsv" --gap 3.0 --opacity 0.6 --legend "connectivity" \
        --title "Cell Type Co-occurrence"

# ── sankey ────────────────────────────────────────────────────────────────────
check "sankey basic" \
    "$BIN" sankey "$DATA/sankey.tsv" --source-col source --target-col target --value-col value \
        --title "Flow Diagram"

check "sankey link-gradient" \
    "$BIN" sankey "$DATA/sankey.tsv" --source-col source --target-col target --value-col value \
        --link-gradient --legend "read flow" --title "Flow Diagram"

# ── phylo ─────────────────────────────────────────────────────────────────────
check "phylo edge-list" \
    "$BIN" phylo "$DATA/phylo.tsv" --parent-col parent --child-col child --length-col length \
        --title "Phylogenetic Tree"

check "phylo newick" \
    "$BIN" phylo --newick "((A:0.1,B:0.2):0.3,C:0.4);" --title "Phylogenetic Tree"

check "phylo circular cladogram" \
    "$BIN" phylo "$DATA/phylo.tsv" --parent-col parent --child-col child --length-col length \
        --branch-style circular --width 800 --height 800 --title "Phylogenetic Tree"

check "phylo circular phylogram" \
    "$BIN" phylo "$DATA/phylo.tsv" --parent-col parent --child-col child --length-col length \
        --branch-style circular --phylogram --width 800 --height 800 --title "Phylogenetic Tree"

# ── synteny ───────────────────────────────────────────────────────────────────
check "synteny basic" \
    "$BIN" synteny "$DATA/synteny_seqs.tsv" \
        --blocks-file "$DATA/synteny_blocks.tsv" --title "Synteny Map"

check "synteny proportional" \
    "$BIN" synteny "$DATA/synteny_seqs.tsv" \
        --blocks-file "$DATA/synteny_blocks.tsv" --proportional --legend "synteny" \
        --title "Synteny Map"

# ── summary ───────────────────────────────────────────────────────────────────
echo ""
echo "Results: $PASS passed, $FAIL failed"
[[ $FAIL -eq 0 ]]
