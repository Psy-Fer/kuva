#!/usr/bin/env bash
# Regenerate terminal output GIFs for the kuva documentation.
#
# Requires: vhs, ttyd, ffmpeg — see CONTRIBUTING.md for installation instructions.
#
# Usage:
#   bash scripts/gen_terminal_docs.sh              # build + record all tapes
#   bash scripts/gen_terminal_docs.sh scatter      # record only docs/tapes/scatter.tape
#   bash scripts/gen_terminal_docs.sh scatter line # record two specific tapes

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TAPES_DIR="$REPO_ROOT/docs/tapes"
OUT_DIR="$REPO_ROOT/docs/src/assets/terminal"

# ── Dependency checks ──────────────────────────────────────────────────────────
for cmd in vhs ttyd ffmpeg; do
    if ! command -v "$cmd" &>/dev/null; then
        echo "error: '$cmd' not found on PATH." >&2
        echo "       See CONTRIBUTING.md — 'Setting up VHS' for installation." >&2
        exit 1
    fi
done

# ── Build release binary ───────────────────────────────────────────────────────
echo "Building release binary..."
cd "$REPO_ROOT"
cargo build --release --bin kuva --features cli --quiet
echo "  -> target/release/kuva"

# ── Output directory ───────────────────────────────────────────────────────────
mkdir -p "$OUT_DIR"

# ── Resolve tapes to run ───────────────────────────────────────────────────────
if [[ $# -ge 1 ]]; then
    TAPES=()
    for name in "$@"; do
        tape="$TAPES_DIR/${name%.tape}.tape"
        if [[ ! -f "$tape" ]]; then
            echo "error: tape not found: $tape" >&2
            exit 1
        fi
        TAPES+=("$tape")
    done
else
    TAPES=("$TAPES_DIR"/*.tape)
fi

# ── Record ─────────────────────────────────────────────────────────────────────
TOTAL=${#TAPES[@]}
COUNT=0
for tape in "${TAPES[@]}"; do
    COUNT=$(( COUNT + 1 ))
    name="$(basename "$tape" .tape)"
    echo "[$COUNT/$TOTAL] $name"
    vhs "$tape"
done

echo ""
echo "Done. GIFs written to $OUT_DIR/"
