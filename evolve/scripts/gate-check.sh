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
# Runs the Invariant suite (run_invariant_suite) against the Variant via the
# g3_invariant harness. Failure → parse the violated invariant name from the
# panic message (e.g. "P3 slot_legality") and append a structured rejection.
if ! G3_OUTPUT=$(cargo test --manifest-path "$WORK_DIR/Cargo.toml" --test g3_invariant 2>&1); then
    echo "G3: FAILED" >&2
    INVARIANT=$(echo "$G3_OUTPUT" | grep -oE 'P[1-5] [a-z_]+' | head -n1)
    INVARIANT="${INVARIANT:-unknown}"
    TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    FINGERPRINT="${VARIANT_FINGERPRINT:-pending}"
    cat >> "$REJECTION_LOG" <<ENTRY

## $TIMESTAMP — cycle=$CYCLE_ID variant=$VARIANT_ID

**Fingerprint**: $FINGERPRINT
**Gate**: G3 (invariant suite)
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
