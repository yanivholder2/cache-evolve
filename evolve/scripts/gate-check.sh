#!/usr/bin/env bash
# G1 + G2 + G3 gates for a variant impl/.
#
# Usage: gate-check <impl-dir> <cycle-id> <variant-id>
#
# Environment:
#   EVOLVE_ROOT   — path to evolve/ directory (default: script's parent dir)
#   REPO_ROOT     — passed through to scripts/build.sh
#   REJECTION_LOG — path to rejection-log.md (default: $EVOLVE_ROOT/rejection-log.md)
#
# Exits 0 if all gates pass. Exits 1 on any gate failure and appends a
# structured entry to rejection-log.md.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EVOLVE_ROOT="${EVOLVE_ROOT:-$(cd "$SCRIPT_DIR/.." && pwd)}"
REJECTION_LOG="${REJECTION_LOG:-$EVOLVE_ROOT/rejection-log.md}"

IMPL_DIR="${1:?Usage: gate-check <impl-dir> <cycle-id> <variant-id>}"
CYCLE_ID="${2:-unknown}"
VARIANT_ID="${3:-unknown}"

MANIFEST="$IMPL_DIR/manifest.toml"

# --- G1: build ---
WORK_DIR=$(bash "$SCRIPT_DIR/build.sh" "$IMPL_DIR" "$CYCLE_ID" "$VARIANT_ID") || exit 1
trap 'rm -rf "$WORK_DIR"' EXIT


# --- G2: runtime contract tests ---
if ! G2_OUTPUT=$(cargo test --manifest-path "$WORK_DIR/Cargo.toml" --test g2_contract 2>&1); then
    echo "G2: FAILED" >&2
    TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    cat >> "$REJECTION_LOG" <<ENTRY

## $TIMESTAMP — cycle=$CYCLE_ID variant=$VARIANT_ID

**Gate**: G2 (trait contract)
**Error**:
\`\`\`
$G2_OUTPUT
\`\`\`
---
ENTRY
    echo "rejection logged to $REJECTION_LOG" >&2
    exit 1
fi

echo "G2: passed" >&2

# --- G3: invariant suite ---
# Parses the violated invariant name from the panic message
# (format: "G3 invariant violated: P[1-5] <snake_name>", set by the harness in
# evolve/harness/g3_invariant.rs and format_err in src/test_utils/invariants.rs).
# If the marker is absent, the cargo failure is a harness error (compile, OOM,
# unrelated panic) — not a variant violation — and is reported as such.
if ! G3_OUTPUT=$(cargo test --manifest-path "$WORK_DIR/Cargo.toml" --test g3_invariant 2>&1); then
    echo "G3: FAILED" >&2
    TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    FINGERPRINT="${VARIANT_FINGERPRINT:-pending}"
    MARKER=$(echo "$G3_OUTPUT" | grep -oE 'G3 invariant violated: P[1-5] [a-z_]+' | head -n1 || true)
    if [[ -n "$MARKER" ]]; then
        INVARIANT="${MARKER#G3 invariant violated: }"
        GATE_LABEL="G3 (invariant suite)"
    else
        echo "warning: G3 cargo test failed without an invariant marker — treating as harness error" >&2
        INVARIANT="<harness error — see Error block>"
        GATE_LABEL="G3 (harness error)"
    fi
    cat >> "$REJECTION_LOG" <<ENTRY

## $TIMESTAMP — cycle=$CYCLE_ID variant=$VARIANT_ID

**Fingerprint**: $FINGERPRINT
**Gate**: $GATE_LABEL
**Failing invariant**: $INVARIANT
**Error**:
\`\`\`
$G3_OUTPUT
\`\`\`
---
ENTRY
    echo "rejection logged to $REJECTION_LOG" >&2
    exit 1
fi

echo "G3: passed" >&2

echo "gate-check: all gates passed for $VARIANT_ID" >&2
