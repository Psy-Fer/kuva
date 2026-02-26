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
    "$BIN" scatter "$DATA/scatter.tsv" --x x --y y

check "scatter color-by" \
    "$BIN" scatter "$DATA/scatter.tsv" --x x --y y --color-by group --legend

check "scatter trend" \
    "$BIN" scatter "$DATA/scatter.tsv" --x x --y y --trend --equation --correlation

# ── line ──────────────────────────────────────────────────────────────────────
check "line color-by" \
    "$BIN" line "$DATA/measurements.tsv" --x time --y value --color-by group

check "line color-by legend" \
    "$BIN" line "$DATA/measurements.tsv" --x time --y value --color-by group --legend

# ── bar ───────────────────────────────────────────────────────────────────────
check "bar basic" \
    "$BIN" bar "$DATA/bar.tsv" --label-col category --value-col count

# ── histogram ─────────────────────────────────────────────────────────────────
check "histogram basic" \
    "$BIN" histogram "$DATA/histogram.tsv" --value-col value

check "histogram bins" \
    "$BIN" histogram "$DATA/histogram.tsv" --value-col value --bins 20

check "histogram normalize" \
    "$BIN" histogram "$DATA/histogram.tsv" --value-col value --normalize

# ── box ───────────────────────────────────────────────────────────────────────
check "box basic" \
    "$BIN" box "$DATA/samples.tsv" --group-col group --value-col expression

check "box strip" \
    "$BIN" box "$DATA/samples.tsv" --group-col group --value-col expression --strip

check "box swarm" \
    "$BIN" box "$DATA/samples.tsv" --group-col group --value-col expression --swarm

# ── violin ────────────────────────────────────────────────────────────────────
check "violin basic" \
    "$BIN" violin "$DATA/samples.tsv" --group-col group --value-col expression

check "violin swarm" \
    "$BIN" violin "$DATA/samples.tsv" --group-col group --value-col expression --swarm

# ── pie ───────────────────────────────────────────────────────────────────────
check "pie basic" \
    "$BIN" pie "$DATA/pie.tsv" --label-col feature --value-col percentage

check "pie donut percent" \
    "$BIN" pie "$DATA/pie.tsv" --label-col feature --value-col percentage --donut --percent --legend

# ── strip ─────────────────────────────────────────────────────────────────────
check "strip jitter" \
    "$BIN" strip "$DATA/samples.tsv" --group-col group --value-col expression

check "strip swarm" \
    "$BIN" strip "$DATA/samples.tsv" --group-col group --value-col expression --swarm

check "strip center" \
    "$BIN" strip "$DATA/samples.tsv" --group-col group --value-col expression --center

# ── waterfall ─────────────────────────────────────────────────────────────────
check "waterfall basic" \
    "$BIN" waterfall "$DATA/waterfall.tsv" --label-col process --value-col log2fc

check "waterfall connectors values" \
    "$BIN" waterfall "$DATA/waterfall.tsv" --label-col process --value-col log2fc --connectors --values

# ── stacked-area ──────────────────────────────────────────────────────────────
check "stacked-area basic" \
    "$BIN" stacked-area "$DATA/stacked_area.tsv" --x-col week --group-col species --y-col abundance

check "stacked-area normalize" \
    "$BIN" stacked-area "$DATA/stacked_area.tsv" --x-col week --group-col species --y-col abundance --normalize

# ── volcano ───────────────────────────────────────────────────────────────────
check "volcano basic" \
    "$BIN" volcano "$DATA/volcano.tsv" --name-col gene --x-col log2fc --y-col pvalue

check "volcano top-n legend" \
    "$BIN" volcano "$DATA/volcano.tsv" --name-col gene --x-col log2fc --y-col pvalue --top-n 10 --legend

# ── manhattan ─────────────────────────────────────────────────────────────────
check "manhattan sequential" \
    "$BIN" manhattan "$DATA/gene_stats.tsv" --chr-col chr --pvalue-col pvalue

check "manhattan top-n" \
    "$BIN" manhattan "$DATA/gene_stats.tsv" --chr-col chr --pvalue-col pvalue --top-n 10

check "manhattan hg38" \
    "$BIN" manhattan "$DATA/gene_stats.tsv" --chr-col chr --pos-col pos --pvalue-col pvalue --genome-build hg38

# ── candlestick ───────────────────────────────────────────────────────────────
check "candlestick basic" \
    "$BIN" candlestick "$DATA/candlestick.tsv" \
        --label-col date --open-col open --high-col high --low-col low --close-col close

check "candlestick volume panel" \
    "$BIN" candlestick "$DATA/candlestick.tsv" \
        --label-col date --open-col open --high-col high --low-col low --close-col close \
        --volume-col volume --volume-panel

# ── heatmap ───────────────────────────────────────────────────────────────────
check "heatmap basic" \
    "$BIN" heatmap "$DATA/heatmap.tsv"

check "heatmap values inferno" \
    "$BIN" heatmap "$DATA/heatmap.tsv" --values --colormap inferno --legend "z-score" --height 800

# ── hist2d ────────────────────────────────────────────────────────────────────
check "hist2d basic" \
    "$BIN" hist2d "$DATA/hist2d.tsv" --x x --y y

check "hist2d fine bins" \
    "$BIN" hist2d "$DATA/hist2d.tsv" --x x --y y --bins-x 30 --bins-y 30 --correlation

# ── contour ───────────────────────────────────────────────────────────────────
check "contour basic" \
    "$BIN" contour "$DATA/contour.tsv" --x x --y y --z density

check "contour filled" \
    "$BIN" contour "$DATA/contour.tsv" --x x --y y --z density --filled --levels 10 --legend "density"

# ── dot ───────────────────────────────────────────────────────────────────────
check "dot basic" \
    "$BIN" dot "$DATA/dot.tsv" --x-col pathway --y-col cell_type \
        --size-col pct_expressed --color-col mean_expr

check "dot legend colorbar" \
    "$BIN" dot "$DATA/dot.tsv" --x-col pathway --y-col cell_type \
        --size-col pct_expressed --color-col mean_expr \
        --size-legend "% expressed" --colorbar "mean expr"

# ── upset ─────────────────────────────────────────────────────────────────────
check "upset basic" \
    "$BIN" upset "$DATA/upset.tsv"

check "upset sort degree" \
    "$BIN" upset "$DATA/upset.tsv" --sort degree --max-visible 10

# ── chord ─────────────────────────────────────────────────────────────────────
check "chord basic" \
    "$BIN" chord "$DATA/chord.tsv"

check "chord gap legend" \
    "$BIN" chord "$DATA/chord.tsv" --gap 3.0 --opacity 0.6 --legend "connectivity"

# ── sankey ────────────────────────────────────────────────────────────────────
check "sankey basic" \
    "$BIN" sankey "$DATA/sankey.tsv" --source-col source --target-col target --value-col value

check "sankey gradient" \
    "$BIN" sankey "$DATA/sankey.tsv" --source-col source --target-col target --value-col value \
        --gradient --legend "read flow"

# ── phylo ─────────────────────────────────────────────────────────────────────
check "phylo edge-list" \
    "$BIN" phylo "$DATA/phylo.tsv" --parent-col parent --child-col child --length-col length

check "phylo newick" \
    "$BIN" phylo --newick "((A:0.1,B:0.2):0.3,C:0.4);"

check "phylo circular cladogram" \
    "$BIN" phylo "$DATA/phylo.tsv" --parent-col parent --child-col child --length-col length \
        --branch-style circular --width 800 --height 800

check "phylo circular phylogram" \
    "$BIN" phylo "$DATA/phylo.tsv" --parent-col parent --child-col child --length-col length \
        --branch-style circular --phylogram --width 800 --height 800

# ── synteny ───────────────────────────────────────────────────────────────────
check "synteny basic" \
    "$BIN" synteny "$DATA/synteny_seqs.tsv" \
        --blocks-file "$DATA/synteny_blocks.tsv"

check "synteny shared-scale" \
    "$BIN" synteny "$DATA/synteny_seqs.tsv" \
        --blocks-file "$DATA/synteny_blocks.tsv" --shared-scale --legend "synteny"

# ── summary ───────────────────────────────────────────────────────────────────
echo ""
echo "Results: $PASS passed, $FAIL failed"
[[ $FAIL -eq 0 ]]
