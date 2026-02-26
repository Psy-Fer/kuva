#!/usr/bin/env bash
# Smoke tests for the visus CLI binary.
# Runs every subcommand against example data and checks that SVG output is produced.
# Usage: ./scripts/smoke_tests.sh [path/to/visus]
# If no binary path is given, uses ./target/debug/visus.

set -euo pipefail

BIN="${1:-./target/debug/visus}"
DATA="./examples/data"

PASS=0
FAIL=0

check() {
    local name="$1"
    shift
    if "$@" | grep -q "<svg"; then
        echo "PASS  $name"
        PASS=$((PASS + 1))
    else
        echo "FAIL  $name"
        FAIL=$((FAIL + 1))
    fi
}

# ── scatter ───────────────────────────────────────────────────────────────────
check "scatter basic" \
    "$BIN" scatter "$DATA/measurements.tsv" --x time --y value

check "scatter color-by" \
    "$BIN" scatter "$DATA/measurements.tsv" --x time --y value --color-by group --legend

check "scatter trend" \
    "$BIN" scatter "$DATA/measurements.tsv" --x time --y value --trend --equation --correlation

# ── line ──────────────────────────────────────────────────────────────────────
check "line basic" \
    "$BIN" line "$DATA/measurements.tsv" --x time --y value

check "line color-by" \
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
    "$BIN" volcano "$DATA/gene_stats.tsv" --name-col gene --x-col log2fc --y-col pvalue

check "volcano top-n legend" \
    "$BIN" volcano "$DATA/gene_stats.tsv" --name-col gene --x-col log2fc --y-col pvalue --top-n 20 --legend

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

# ── summary ───────────────────────────────────────────────────────────────────
echo ""
echo "Results: $PASS passed, $FAIL failed"
[[ $FAIL -eq 0 ]]
