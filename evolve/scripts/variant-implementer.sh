#!/usr/bin/env bash
# Variant implementer — default deterministic stub.
#
# Real implementations call the variant-implementer subagent
# (.claude/agents/variant-implementer.md). For the walking skeleton we copy the
# good fixture so the cohort fan-out and downstream gates can be exercised
# end-to-end without depending on a live LLM.
#
# Override by setting VARIANT_IMPLEMENTER to a different command in the
# orchestrator environment.
#
# Usage: variant-implementer.sh <hypothesis-file> <output-impl-dir>
set -euo pipefail

HYPOTHESIS_FILE="${1:?Usage: variant-implementer.sh <hypothesis-file> <output-impl-dir>}"
OUT_IMPL="${2:?Usage: variant-implementer.sh <hypothesis-file> <output-impl-dir>}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EVOLVE_ROOT="${EVOLVE_ROOT:-$(cd "$SCRIPT_DIR/.." && pwd)}"
GOOD_IMPL="$EVOLVE_ROOT/fixtures/good/impl"

if [ ! -f "$HYPOTHESIS_FILE" ]; then
    echo "error: hypothesis file not found: $HYPOTHESIS_FILE" >&2
    exit 1
fi

mkdir -p "$OUT_IMPL"
cp "$GOOD_IMPL/segment.rs"    "$OUT_IMPL/segment.rs"
cp "$GOOD_IMPL/manifest.toml" "$OUT_IMPL/manifest.toml"
