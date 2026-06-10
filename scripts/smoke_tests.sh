#!/usr/bin/env bash
# Smoke tests for the kuva CLI binary.
# Runs every subcommand against example data and checks that SVG output is produced.
#
# Usage:
#   ./scripts/smoke_tests.sh [path/to/kuva] [--save [dir]]
#
# Options:
#   First non-flag arg   Path to kuva binary (default: ./target/debug/kuva)
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

BIN="${BIN:-./target/debug/kuva}"
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

# Passes when the command exits non-zero (error path test).
check_error() {
    local name="$1"
    shift
    if ! "$@" > /dev/null 2>&1; then
        echo "PASS  $name"
        PASS=$((PASS + 1))
    else
        echo "FAIL  $name  (expected error, got success)"
        FAIL=$((FAIL + 1))
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

check "scatter tick format sci" \
    "$BIN" scatter "$DATA/scatter.tsv" --x x --y y \
        --y-tick-format sci --title "Scatter Y=sci"

check "scatter tick format fixed" \
    "$BIN" scatter "$DATA/scatter.tsv" --x x --y y \
        --x-tick-format fixed:2 --y-tick-format fixed:3 --title "Scatter fixed ticks"

check "line tick format int" \
    "$BIN" line "$DATA/measurements.tsv" --x time --y value --color-by group \
        --y-tick-format int --title "Line Y=int"

check "line tick format percent" \
    "$BIN" line "$DATA/measurements.tsv" --x time --y value --color-by group \
        --y-tick-format percent --title "Line Y=percent"

check "scatter multi-y two columns" \
    "$BIN" scatter "$DATA/measurements.tsv" --x time --y value,time --legend \
        --title "Scatter Multi-Y (2 cols)" --x-label "Time" --y-label "Value"

check "scatter multi-y three columns no legend" \
    "$BIN" scatter "$DATA/measurements.tsv" --x time --y value,time,value \
        --title "Scatter Multi-Y (3 cols)"

check_error "scatter multi-y color-by conflict" \
    "$BIN" scatter "$DATA/measurements.tsv" --x time --y value,time --color-by group

# ── line ──────────────────────────────────────────────────────────────────────
check "line color-by" \
    "$BIN" line "$DATA/measurements.tsv" --x time --y value --color-by group \
        --title "Growth Curves" --x-label "Time" --y-label "Value"

check "line color-by legend" \
    "$BIN" line "$DATA/measurements.tsv" --x time --y value --color-by group --legend \
        --title "Growth Curves" --x-label "Time" --y-label "Value"

check "line multi-y two columns" \
    "$BIN" line "$DATA/measurements.tsv" --x time --y value,time --legend \
        --title "Line Multi-Y (2 cols)" --x-label "Time" --y-label "Value"

check "line multi-y with fill" \
    "$BIN" line "$DATA/measurements.tsv" --x time --y value,time --fill --legend \
        --title "Line Multi-Y Filled" --x-label "Time" --y-label "Value"

check "line multi-y dashed" \
    "$BIN" line "$DATA/measurements.tsv" --x time --y value,time --dashed \
        --title "Line Multi-Y Dashed"

check_error "line multi-y color-by conflict" \
    "$BIN" line "$DATA/measurements.tsv" --x time --y value,time --color-by group

# ── bar ───────────────────────────────────────────────────────────────────────
check "bar basic" \
    "$BIN" bar "$DATA/bar.tsv" --label-col category --value-col count \
        --title "Category Counts" --x-label "Category" --y-label "Count"

check "bar count-by" \
    "$BIN" bar "$DATA/scatter.tsv" --count-by group \
        --title "Points per Group" --x-label "Group" --y-label "Count"

check "bar agg sum" \
    "$BIN" bar "$DATA/stacked_area.tsv" --label-col species --value-col abundance --agg sum \
        --title "Total Abundance per Species" --x-label "Species" --y-label "Total Abundance"

check "bar agg mean" \
    "$BIN" bar "$DATA/stacked_area.tsv" --label-col species --value-col abundance --agg mean \
        --title "Mean Abundance per Species" --x-label "Species" --y-label "Mean Abundance"

check "bar agg median" \
    "$BIN" bar "$DATA/stacked_area.tsv" --label-col species --value-col abundance --agg median \
        --title "Median Abundance per Species" --x-label "Species" --y-label "Median Abundance"

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

check "histogram bin-aligned 6 bins" \
    "$BIN" histogram "$DATA/histogram.tsv" --value-col value --bins 6 \
        --title "Bin-Aligned Ticks (6 bins)" --x-label "Value" --y-label "Count"

check "histogram bin-aligned 7 bins normalize" \
    "$BIN" histogram "$DATA/histogram.tsv" --value-col value --bins 7 --normalize \
        --title "Bin-Aligned Ticks (7 bins)" --x-label "Value" --y-label "Density"

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

check "box group-colors" \
    "$BIN" box "$DATA/samples.tsv" --group-col group --value-col expression \
        --group-colors "steelblue,tomato,seagreen,goldenrod,mediumpurple" \
        --title "Expression by Group (colored)" --x-label "Group" --y-label "Expression"

# ── violin ────────────────────────────────────────────────────────────────────
check "violin basic" \
    "$BIN" violin "$DATA/samples.tsv" --group-col group --value-col expression \
        --title "Expression Distribution" --x-label "Group" --y-label "Expression"

check "violin overlay-swarm" \
    "$BIN" violin "$DATA/samples.tsv" --group-col group --value-col expression --overlay-swarm \
        --title "Expression Distribution" --x-label "Group" --y-label "Expression"

check "violin group-colors" \
    "$BIN" violin "$DATA/samples.tsv" --group-col group --value-col expression \
        --group-colors "steelblue,tomato,seagreen,goldenrod,mediumpurple" \
        --title "Expression Distribution (colored)" --x-label "Group" --y-label "Expression"

# ── pie ───────────────────────────────────────────────────────────────────────
check "pie basic" \
    "$BIN" pie "$DATA/pie.tsv" --label-col feature --value-col percentage \
        --title "Genome Composition"

check "pie count-by" \
    "$BIN" pie "$DATA/scatter.tsv" --count-by group --percent --legend \
        --title "Group Proportions"

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

# ── forest ────────────────────────────────────────────────────────────────────
check "forest basic" \
    "$BIN" forest "$DATA/forest.tsv" --label-col study --estimate-col estimate \
        --ci-lower-col ci_lower --ci-upper-col ci_upper \
        --title "Meta-Analysis" --x-label "Effect Size"

check "forest weighted" \
    "$BIN" forest "$DATA/forest.tsv" --label-col study --estimate-col estimate \
        --ci-lower-col ci_lower --ci-upper-col ci_upper --weight-col weight \
        --title "Meta-Analysis (Weighted)" --x-label "Effect Size"

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

check "volcano pvalue-col-is-log" \
    "$BIN" volcano "$DATA/volcano_logp.tsv" --name-col gene --x-col log2fc --y-col neg_log10_pvalue --pvalue-col-is-log \
        --title "Differential Expression (log p input)" --x-label "log2 Fold Change" "--y-label=-log10(p-value)"

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

check "manhattan pvalue-col-is-log" \
    "$BIN" manhattan "$DATA/gene_stats_logp.tsv" --chr-col chr --pvalue-col neg_log10_pvalue --pvalue-col-is-log \
        --title "GWAS Results (log p input)" --x-label "Chromosome" "--y-label=-log10(p-value)"

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

check "heatmap long-format" \
    "$BIN" heatmap "$DATA/stacked_area.tsv" --long-format \
        --row-col species --col-col week --value-col abundance \
        --title "Abundance by Species and Week" --x-label "Week" --y-label "Species"

# ── hist2d ────────────────────────────────────────────────────────────────────
check "hist2d basic" \
    "$BIN" hist2d "$DATA/hist2d.tsv" --x x --y y \
        --title "2D Density" --x-label "X" --y-label "Y"

check "hist2d fine bins" \
    "$BIN" hist2d "$DATA/hist2d.tsv" --x x --y y --bins-x 30 --bins-y 30 --correlation \
        --title "2D Density" --x-label "X" --y-label "Y"

check "hist2d explicit range clips outliers" \
    "$BIN" hist2d "$DATA/hist2d.tsv" --x x --y y --bins-x 20 --bins-y 20 \
        --x-min 20 --x-max 50 --y-min 20 --y-max 50 \
        --title "hist2d clipped range" --x-label "X" --y-label "Y"

check "hist2d turbo colormap" \
    "$BIN" hist2d "$DATA/hist2d.tsv" --x x --y y --bins-x 20 --bins-y 20 \
        --colormap turbo --title "hist2d turbo" --x-label "X" --y-label "Y"

check "hist2d log-count" \
    "$BIN" hist2d "$DATA/hist2d.tsv" --x x --y y --bins-x 20 --bins-y 20 \
        --log-count --title "hist2d log count" --x-label "X" --y-label "Y"

check "hist2d colorbar sci format" \
    "$BIN" hist2d "$DATA/hist2d.tsv" --x x --y y --bins-x 20 --bins-y 20 \
        --colorbar-tick-format sci --title "hist2d sci colorbar" --x-label "X" --y-label "Y"

# ── hexbin ────────────────────────────────────────────────────────────────────
check "hexbin basic" \
    "$BIN" hexbin "$DATA/hexbin.tsv" --x x --y y \
        --title "Hexbin Plot" --x-label "X" --y-label "Y"

check "hexbin n-bins and colormap" \
    "$BIN" hexbin "$DATA/hexbin.tsv" --x x --y y \
        --n-bins 15 --colormap inferno \
        --title "Hexbin Inferno" --x-label "X" --y-label "Y"

check "hexbin z-mean flat-top" \
    "$BIN" hexbin "$DATA/hexbin.tsv" --x x --y y --z z --reduce mean \
        --flat-top --stroke "#444444" \
        --title "Hexbin Z Mean" --x-label "X" --y-label "Y"

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

# ── network ───────────────────────────────────────────────────────────────────
check "network basic" \
    "$BIN" network "$DATA/network.tsv" --source-col source --target-col target \
        --labels --title "Gene Regulatory Network"

check "network directed weighted legend" \
    "$BIN" network "$DATA/network.tsv" --source-col source --target-col target \
        --weight-col weight --group-col group --directed --labels \
        --legend "pathway" --title "Gene Regulatory Network"

check "network circle layout" \
    "$BIN" network "$DATA/network.tsv" --source-col source --target-col target \
        --layout circle --labels --title "Circle Layout"

check "network kk layout" \
    "$BIN" network "$DATA/network.tsv" --source-col source --target-col target \
        --layout kk --labels --title "Kamada-Kawai Layout"

check "network matrix" \
    "$BIN" network "$DATA/network_matrix.tsv" --matrix --directed --labels \
        --title "Matrix Input"

# ── sankey ────────────────────────────────────────────────────────────────────
check "sankey basic" \
    "$BIN" sankey "$DATA/sankey.tsv" --source-col source --target-col target --value-col value \
        --title "Flow Diagram"

check "sankey link-gradient" \
    "$BIN" sankey "$DATA/sankey.tsv" --source-col source --target-col target --value-col value \
        --link-gradient --legend "read flow" --title "Flow Diagram"

check "sankey flow-labels" \
    "$BIN" sankey "$DATA/sankey.tsv" --source-col source --target-col target --value-col value \
        --flow-labels --title "Flow Labels"

check "sankey flow-percent" \
    "$BIN" sankey "$DATA/sankey.tsv" --source-col source --target-col target --value-col value \
        --flow-percent --title "Flow Percent"

check "sankey flow-labels-unit" \
    "$BIN" sankey "$DATA/sankey.tsv" --source-col source --target-col target --value-col value \
        --flow-labels --flow-label-unit reads --flow-label-min-height 0 --title "Flow Labels Unit"

check "sankey flow-labels-sci" \
    "$BIN" sankey "$DATA/sankey.tsv" --source-col source --target-col target --value-col value \
        --flow-labels --flow-label-format sci --title "Flow Labels Sci"

check "sankey alluvium crossings left-coloring" \
    "$BIN" sankey "$DATA/sankey_alluvium.tsv" \
        --axis-col tissue --axis-col cluster --axis-col sex --value-col count \
        --node-order crossings --node-order-seed 42 --coloring left \
        --title "Alluvium Crossings"

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

# ── density ───────────────────────────────────────────────────────────────────
check "density basic" \
    "$BIN" density "$DATA/samples.tsv" \
        --value expression --x-label "Expression" --y-label "Density" \
        --title "Density"

check "density filled color-by" \
    "$BIN" density "$DATA/samples.tsv" \
        --value expression --color-by group --filled \
        --title "Density by group"

check "density x-range bounded" \
    "$BIN" density "$DATA/samples.tsv" \
        --value expression --x-min 0 --x-max 10 \
        --x-label "Expression" --y-label "Density" \
        --title "Density bounded range"

# ── ecdf ──────────────────────────────────────────────────────────────────────
check "ecdf basic" \
    "$BIN" ecdf "$DATA/samples.tsv" \
        --value expression --x-label "Expression" --y-label "F(x)" \
        --title "ECDF"

check "ecdf color-by with confidence band" \
    "$BIN" ecdf "$DATA/samples.tsv" \
        --value expression --color-by group --confidence-band \
        --title "ECDF by group"

check "ecdf complementary with rug" \
    "$BIN" ecdf "$DATA/samples.tsv" \
        --value expression --complementary --rug \
        --x-label "Expression" --y-label "1 - F(x)" \
        --title "CCDF with rug"

check "ecdf percentile lines and markers" \
    "$BIN" ecdf "$DATA/samples.tsv" \
        --value expression --percentile-lines 0.25,0.5,0.75 --markers \
        --title "ECDF with percentiles"

check "ecdf smooth" \
    "$BIN" ecdf "$DATA/samples.tsv" \
        --value expression --color-by group --smooth \
        --title "Smooth ECDF"

# ── ridgeline ─────────────────────────────────────────────────────────────────
check "ridgeline basic" \
    "$BIN" ridgeline "$DATA/samples.tsv" \
        --group-by group --value expression \
        --title Ridgeline --x-label Expression --y-label Group

check "ridgeline overlap" \
    "$BIN" ridgeline "$DATA/samples.tsv" \
        --group-by group --value expression --overlap 1.0

# ── synteny ───────────────────────────────────────────────────────────────────
check "synteny basic" \
    "$BIN" synteny "$DATA/synteny_seqs.tsv" \
        --blocks-file "$DATA/synteny_blocks.tsv" --title "Synteny Map"

check "synteny proportional" \
    "$BIN" synteny "$DATA/synteny_seqs.tsv" \
        --blocks-file "$DATA/synteny_blocks.tsv" --proportional --legend "synteny" \
        --title "Synteny Map"

# ── radar ──────────────────────────────────────────────────────────────────────
check "radar basic" \
    "$BIN" radar "$DATA/radar.tsv" \
        --axes Sensitivity Specificity Precision F1 AUC \
        --color-by tool --legend \
        --title "Radar Chart"

check "radar filled" \
    "$BIN" radar "$DATA/radar.tsv" \
        --axes Sensitivity Specificity Precision F1 AUC \
        --color-by tool --filled --legend \
        --title "Radar Chart Filled"

check "radar normalize" \
    "$BIN" radar "$DATA/radar.tsv" \
        --axes Sensitivity Specificity Precision F1 AUC \
        --color-by tool --normalize --legend \
        --title "Radar Normalized"

# ── polar ──────────────────────────────────────────────────────────────────────
check "polar basic" \
    "$BIN" polar "$DATA/polar.tsv" --r r --theta theta \
        --title "Polar Plot"

check "polar color-by" \
    "$BIN" polar "$DATA/polar.tsv" --r r --theta theta --color-by group \
        --title "Polar Plot"

# ── ternary ────────────────────────────────────────────────────────────────────
check "ternary basic" \
    "$BIN" ternary "$DATA/ternary.tsv" --a a --b b --c c \
        --title "Ternary Plot"

check "ternary color-by" \
    "$BIN" ternary "$DATA/ternary.tsv" --a a --b b --c c --color-by group \
        --a-label "Silicon" --b-label "Oxygen" --c-label "Carbon" \
        --title "Ternary Composition"

check "ternary normalize" \
    "$BIN" ternary "$DATA/ternary.tsv" --a a --b b --c c --color-by group \
        --normalize \
        --title "Ternary Normalized"

check "ternary fine grid" \
    "$BIN" ternary "$DATA/ternary.tsv" --a a --b b --c c \
        --grid-lines 10 \
        --title "Ternary Fine Grid"

check "ternary coarse grid" \
    "$BIN" ternary "$DATA/ternary.tsv" --a a --b b --c c \
        --grid-lines 4 \
        --title "Ternary Coarse Grid"

check "ternary legend" \
    "$BIN" ternary "$DATA/ternary.tsv" --a a --b b --c c --color-by group \
        --legend \
        --title "Ternary Legend"

# ── scatter3d ─────────────────────────────────────────────────────────────────
check "scatter3d basic" \
    "$BIN" scatter3d "$DATA/scatter3d.tsv" --x x --y y --z z \
        --title "3D Scatter" --x-label "X" --y-label "Y" --z-label "Z"

check "scatter3d color-by" \
    "$BIN" scatter3d "$DATA/scatter3d.tsv" --x x --y y --z z --color-by group \
        --title "3D Scatter Grouped"

check "scatter3d z-colormap" \
    "$BIN" scatter3d "$DATA/scatter3d.tsv" --x x --y y --z z \
        --z-color viridis \
        --title "3D Scatter Z-Color"

check "scatter3d depth-shade" \
    "$BIN" scatter3d "$DATA/scatter3d.tsv" --x x --y y --z z \
        --depth-shade \
        --title "3D Scatter Depth Shade"

check "scatter3d no-grid no-box" \
    "$BIN" scatter3d "$DATA/scatter3d.tsv" --x x --y y --z z \
        --no-grid --no-box \
        --title "3D Scatter No Grid"

check "scatter3d alternate view" \
    "$BIN" scatter3d "$DATA/scatter3d.tsv" --x x --y y --z z \
        --azimuth -120 --elevation 20 \
        --title "3D Scatter Alt View"

# ── surface3d ─────────────────────────────────────────────────────────────────
check "surface3d basic" \
    "$BIN" surface3d "$DATA/surface3d.tsv" --x x --y y --z z \
        --z-color viridis \
        --title "3D Surface" --x-label "X" --y-label "Y" --z-label "Z"

check "surface3d high-res" \
    "$BIN" surface3d "$DATA/surface3d.tsv" --x x --y y --z z \
        --z-color inferno --resolution 20 \
        --title "3D Surface (Upsampled)"

check "surface3d no-wireframe" \
    "$BIN" surface3d "$DATA/surface3d.tsv" --x x --y y --z z \
        --z-color viridis --no-wireframe \
        --title "3D Surface No Wireframe"

check "surface3d alpha" \
    "$BIN" surface3d "$DATA/surface3d.tsv" --x x --y y --z z \
        --alpha 0.7 --color steelblue \
        --title "3D Surface Alpha"

check "surface3d no-grid no-box" \
    "$BIN" surface3d "$DATA/surface3d.tsv" --x x --y y --z z \
        --z-color viridis --no-grid --no-box \
        --title "3D Surface No Grid"

check "surface3d alternate view" \
    "$BIN" surface3d "$DATA/surface3d.tsv" --x x --y y --z z \
        --z-color viridis --azimuth 45 --elevation 45 \
        --title "3D Surface Alt View"

# ── qq ───────────────────────────────────────────────────────────────────────
check "qq normal basic" \
    "$BIN" qq "$DATA/samples.tsv" \
        --value expression \
        --title "Normal Q-Q"

check "qq normal multigroup" \
    "$BIN" qq "$DATA/samples.tsv" \
        --value expression --color-by group \
        --title "Multi-group Normal Q-Q"

check "qq genomic basic" \
    "$BIN" qq "$DATA/gene_stats.tsv" \
        --value pvalue --genomic \
        --title "Genomic Q-Q"

check "qq genomic with ci band and lambda" \
    "$BIN" qq "$DATA/gene_stats.tsv" \
        --value pvalue --genomic --ci-band --lambda \
        --title "Genomic Q-Q with CI and lambda"

# ── streamgraph ───────────────────────────────────────────────────────────────
check "streamgraph wiggle (default)" \
    "$BIN" streamgraph "$DATA/streamgraph.tsv" \
        --title "Microbiome streamgraph"

check "streamgraph symmetric" \
    "$BIN" streamgraph "$DATA/streamgraph.tsv" \
        --baseline symmetric \
        --title "Symmetric streamgraph"

check "streamgraph normalized" \
    "$BIN" streamgraph "$DATA/streamgraph.tsv" \
        --normalize \
        --title "Normalised streamgraph"

check "streamgraph linear with stroke" \
    "$BIN" streamgraph "$DATA/streamgraph.tsv" \
        --linear --stroke \
        --title "Linear streamgraph"

# ── interactive ───────────────────────────────────────────────────────────────
check "scatter interactive" \
    "$BIN" scatter "$DATA/scatter.tsv" --x x --y y \
        --color-by group --legend \
        --interactive \
        --title "Interactive Scatter"

# ── text wrapping ─────────────────────────────────────────────────────────────
check "wrap title" \
    "$BIN" scatter "$DATA/scatter.tsv" --x x --y y \
        --title "This is a deliberately long title that should definitely wrap onto multiple lines when wrap is set" \
        --wrap 30

check "wrap legend" \
    "$BIN" scatter "$DATA/scatter.tsv" --x x --y y \
        --color-by group --legend \
        --legend-wrap 15 \
        --title "Legend Wrap"

check "wrap x-label" \
    "$BIN" scatter "$DATA/scatter.tsv" --x x --y y \
        --x-label "A very long x-axis label that would normally make the bottom margin huge" \
        --x-label-wrap 25

check "wrap y-label" \
    "$BIN" scatter "$DATA/scatter.tsv" --x x --y y \
        --y-label "A very long y-axis label that wraps into multiple rotated lines" \
        --y-label-wrap 20

check "wrap dark theme" \
    "$BIN" scatter "$DATA/scatter.tsv" --x x --y y \
        --title "Long dark theme title that should wrap nicely" \
        --wrap 25 --theme dark

check "wrap with scale" \
    "$BIN" scatter "$DATA/scatter.tsv" --x x --y y --color-by group --legend \
        --title "Scaled wrapped plot" \
        --wrap 20 --scale 1.5

# ── slope ─────────────────────────────────────────────────────────────────────
check "slope basic" \
    "$BIN" slope "$DATA/slope.tsv" \
        --label-col label --before-col before --after-col after \
        --before-label "Before" --after-label "After" \
        --title "Weight Loss by Diet"

check "slope direction colors" \
    "$BIN" slope "$DATA/slope.tsv" \
        --label-col label --before-col before --after-col after \
        --show-values --title "Slope with Values"

# ── lollipop ──────────────────────────────────────────────────────────────────
check "lollipop basic" \
    "$BIN" lollipop "$DATA/lollipop.tsv" \
        --x-col gene --y-col expression \
        --label-col gene \
        --title "Gene Expression"

check "lollipop styled" \
    "$BIN" lollipop "$DATA/lollipop.tsv" \
        --x-col gene --y-col expression \
        --color steelblue --dot-radius 6 \
        --legend "Expression" --title "Styled Lollipop"

# ── raincloud ─────────────────────────────────────────────────────────────────
check "raincloud basic" \
    "$BIN" raincloud "$DATA/raincloud.tsv" \
        --group-col group --value-col value \
        --title "Raincloud Plot"

check "raincloud no rain" \
    "$BIN" raincloud "$DATA/raincloud.tsv" \
        --group-col group --value-col value \
        --no-rain --legend "group" --title "Cloud + Box Only"

# ── mosaic ────────────────────────────────────────────────────────────────────
check "mosaic basic" \
    "$BIN" mosaic "$DATA/mosaic.tsv" \
        --col-col region --row-col outcome --value-col count \
        --title "Outcomes by Region"

check "mosaic with values" \
    "$BIN" mosaic "$DATA/mosaic.tsv" \
        --col-col region --row-col outcome --value-col count \
        --show-values --title "Mosaic with Values"

# ── waffle ────────────────────────────────────────────────────────────────────
check "waffle basic" \
    "$BIN" waffle "$DATA/waffle.tsv" \
        --label-col category --value-col value --color-col color \
        --legend "Energy Mix" --title "Energy Sources"

check "waffle circle shape" \
    "$BIN" waffle "$DATA/waffle.tsv" \
        --label-col category --value-col value --color-col color \
        --shape circle --show-percents --title "Waffle Circles"

# ── pyramid ───────────────────────────────────────────────────────────────────
check "pyramid basic" \
    "$BIN" pyramid "$DATA/pyramid.tsv" \
        --label-col age --left-col male --right-col female \
        --left-label "Male" --right-label "Female" \
        --title "Population Pyramid"

check "pyramid normalized" \
    "$BIN" pyramid "$DATA/pyramid.tsv" \
        --label-col age --left-col male --right-col female \
        --left-label "Male" --right-label "Female" \
        --normalize --legend --title "Normalized Pyramid"

# ── roc ───────────────────────────────────────────────────────────────────────
check "roc basic" \
    "$BIN" roc "$DATA/roc.tsv" \
        --score-col score --label-col label \
        --auc-label --title "ROC Curve"

check "roc with ci" \
    "$BIN" roc "$DATA/roc.tsv" \
        --score-col score --label-col label \
        --ci --auc-label --legend "Model" --title "ROC with CI"

# ── pr ────────────────────────────────────────────────────────────────────────
check "pr basic" \
    "$BIN" pr "$DATA/pr.tsv" \
        --score-col score --label-col label \
        --auc-label --title "Precision-Recall Curve"

check "pr no baseline" \
    "$BIN" pr "$DATA/pr.tsv" \
        --score-col score --label-col label \
        --no-baseline --legend "Classifier" --title "PR no Baseline"

# ── survival ──────────────────────────────────────────────────────────────────
check "survival basic" \
    "$BIN" survival "$DATA/survival.tsv" \
        --time-col time --event-col event --group-col group \
        --title "Kaplan-Meier Survival"

check "survival no ci" \
    "$BIN" survival "$DATA/survival.tsv" \
        --time-col time --event-col event --group-col group \
        --no-ci --legend "Group" --title "KM no CI"

# ── horizon ───────────────────────────────────────────────────────────────────
check "horizon basic" \
    "$BIN" horizon "$DATA/horizon.tsv" \
        --x-col week --value-col value --group-col series \
        --title "Horizon Chart"

check "horizon with value labels" \
    "$BIN" horizon "$DATA/horizon.tsv" \
        --x-col week --value-col value --group-col series \
        --value-labels --n-bands 4 --title "Horizon 4 Bands"

# ── parallel ──────────────────────────────────────────────────────────────────
check "parallel basic" \
    "$BIN" parallel "$DATA/parallel.tsv" \
        --value-cols sepal_length sepal_width petal_length petal_width \
        --group-col species \
        --title "Parallel Coordinates"

check "parallel curved" \
    "$BIN" parallel "$DATA/parallel.tsv" \
        --value-cols sepal_length sepal_width petal_length petal_width \
        --group-col species \
        --curved --show-mean --legend "Species" --title "Parallel Curved"

# ── venn ──────────────────────────────────────────────────────────────────────
check "venn basic" \
    "$BIN" venn "$DATA/venn.tsv" \
        --element-col element --set-col set \
        --title "Venn Diagram"

check "venn proportional" \
    "$BIN" venn "$DATA/venn.tsv" \
        --element-col element --set-col set \
        --proportional --legend "Gene Sets" --title "Proportional Venn"

# ── calendar ──────────────────────────────────────────────────────────────────
check "calendar basic" \
    "$BIN" calendar "$DATA/calendar.tsv" \
        --date-col date --value-col count \
        --title "Calendar Heatmap"

check "calendar date range" \
    "$BIN" calendar "$DATA/calendar.tsv" \
        --date-col date --value-col count \
        --start 2024-01-01 --end 2024-06-30 \
        --agg sum --title "Calendar Range"

# ── bump ──────────────────────────────────────────────────────────────────────
check "bump basic" \
    "$BIN" bump "$DATA/bump.tsv" \
        --series series --time time --rank rank \
        --title "Ranking Over Time"

check "bump with legend" \
    "$BIN" bump "$DATA/bump.tsv" \
        --series series --time time --rank rank \
        --title "Bump Chart"

# ── funnel ────────────────────────────────────────────────────────────────────
check "funnel basic" \
    "$BIN" funnel "$DATA/funnel.tsv" \
        --label stage --value n_screened \
        --title "Clinical Trial Funnel"

check "funnel diverging" \
    "$BIN" funnel "$DATA/funnel.tsv" \
        --label stage --value n_screened --mirror-col n_placebo \
        --left-label "Treatment" --right-label "Placebo" \
        --title "Diverging Funnel"

# ── rose ──────────────────────────────────────────────────────────────────────
check "rose basic" \
    "$BIN" rose "$DATA/rose.tsv" \
        --label direction --value high_speed \
        --title "Wind Rose"

check "rose grouped" \
    "$BIN" rose "$DATA/rose.tsv" \
        --label direction --value high_speed \
        --legend "Speed" --title "Wind Rose"

# ── treemap ───────────────────────────────────────────────────────────────────
check "treemap basic" \
    "$BIN" treemap "$DATA/treemap.tsv" \
        --label label --value value \
        --title "World Population"

check "treemap hierarchical" \
    "$BIN" treemap "$DATA/treemap.tsv" \
        --label label --value value --parent parent \
        --title "World Population by Region"

# ── sunburst ──────────────────────────────────────────────────────────────────
check "sunburst basic" \
    "$BIN" sunburst "$DATA/sunburst.tsv" \
        --label label --value value \
        --title "Animal Kingdom"

check "sunburst hierarchical" \
    "$BIN" sunburst "$DATA/sunburst.tsv" \
        --label label --value value --parent parent \
        --title "Animal Kingdom by Class"

# ── quiver ────────────────────────────────────────────────────────────────────
check "quiver basic" \
    "$BIN" quiver "$DATA/quiver.tsv" --no-grid \
        --title "Quiver Field" --x-label "x" --y-label "y"

check "quiver with colormap" \
    "$BIN" quiver "$DATA/quiver.tsv" --no-grid \
        --colormap viridis --colorbar-label "Speed" \
        --title "Quiver Magnitude"

check "quiver pivot middle" \
    "$BIN" quiver "$DATA/quiver.tsv" --no-grid \
        --pivot middle --tight-bounds \
        --title "Quiver Pivot Middle"

check "quiver pivot tip" \
    "$BIN" quiver "$DATA/quiver.tsv" --no-grid \
        --pivot tip --arrow-scale 0.3 \
        --title "Quiver Pivot Tip"

check "quiver explicit head" \
    "$BIN" quiver "$DATA/quiver.tsv" --no-grid \
        --head-length 12 --head-width 4 --shaft-width 2 \
        --title "Quiver Fixed Head"

check "quiver with legend" \
    "$BIN" quiver "$DATA/quiver.tsv" --no-grid \
        --legend "wind" \
        --title "Quiver Legend"

check "quiver grid on + tight bounds" \
    "$BIN" quiver "$DATA/quiver.tsv" \
        --tight-bounds --pivot middle \
        --title "Quiver Grid + Clip"

# ── summary ───────────────────────────────────────────────────────────────────
echo ""
echo "Results: $PASS passed, $FAIL failed"
[[ $FAIL -eq 0 ]]
