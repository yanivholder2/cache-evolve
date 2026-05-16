#!/usr/bin/env bash
# G1 gate: compile a variant impl/ into a test artifact.
#
# Usage: build <impl-dir> <cycle-id> <variant-id>
#
# Environment:
#   EVOLVE_ROOT   — path to evolve/ directory (default: script's parent dir)
#   REPO_ROOT     — path to the dash crate root (default: $EVOLVE_ROOT/..)
#   REJECTION_LOG — path to rejection-log.md (default: $EVOLVE_ROOT/rejection-log.md)
#
# On success: prints the temp workspace path to stdout and exits 0.
#             The caller is responsible for cleaning up the workspace directory.
# On failure: appends a structured entry to rejection-log.md and exits 1.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EVOLVE_ROOT="${EVOLVE_ROOT:-$(cd "$SCRIPT_DIR/.." && pwd)}"
REPO_ROOT="${REPO_ROOT:-$(cd "$EVOLVE_ROOT/.." && pwd)}"
REJECTION_LOG="${REJECTION_LOG:-$EVOLVE_ROOT/rejection-log.md}"

IMPL_DIR="${1:?Usage: build <impl-dir> <cycle-id> <variant-id>}"
CYCLE_ID="${2:-unknown}"
VARIANT_ID="${3:-unknown}"

IMPL_DIR="$(cd "$IMPL_DIR" && pwd)"
SEGMENT_RS="$IMPL_DIR/segment.rs"

if [[ ! -f "$SEGMENT_RS" ]]; then
    echo "error: $SEGMENT_RS not found — impl/ must contain segment.rs" >&2
    exit 1
fi

WORK_DIR=$(mktemp -d -t "dash-variant-XXXXXX")

cat > "$WORK_DIR/Cargo.toml" <<EOF
[package]
name = "variant-check"
version = "0.1.0"
edition = "2021"

[dependencies]
dash = { package = "dash-cache-rs", path = "$REPO_ROOT", features = ["test-utils"] }
EOF

mkdir -p "$WORK_DIR/src" "$WORK_DIR/tests"

cat > "$WORK_DIR/src/lib.rs" <<'EOF'
pub mod variant;
pub use variant::VariantSegment;
EOF

cp "$SEGMENT_RS" "$WORK_DIR/src/variant.rs"

if [[ -f "$IMPL_DIR/manifest.toml" ]]; then
    cp "$IMPL_DIR/manifest.toml" "$WORK_DIR/manifest.toml"
fi

cp "$SCRIPT_DIR/../harness/g2_contract.rs" "$WORK_DIR/tests/g2_contract.rs"
cp "$SCRIPT_DIR/../harness/g3_invariant.rs" "$WORK_DIR/tests/g3_invariant.rs"

echo "G1: compiling variant $VARIANT_ID ..." >&2
if ! CARGO_OUTPUT=$(cargo build --tests --manifest-path "$WORK_DIR/Cargo.toml" 2>&1); then
    echo "G1: FAILED" >&2
    TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    cat >> "$REJECTION_LOG" <<ENTRY

## $TIMESTAMP — cycle=$CYCLE_ID variant=$VARIANT_ID

**Gate**: G1 (build)
**Error**:
\`\`\`
$CARGO_OUTPUT
\`\`\`
---
ENTRY
    echo "rejection logged to $REJECTION_LOG" >&2
    rm -rf "$WORK_DIR"
    exit 1
fi

echo "G1: passed" >&2
echo "$WORK_DIR"
