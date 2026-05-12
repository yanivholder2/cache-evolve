#!/usr/bin/env bash
# Stub orchestrator — walking-skeleton implementation of one hardcoded cycle.
# Every subagent slot is a stub: fixed variant, all gates pass, fixed fitness numbers.
#
# Usage: [HYPOTHESIS="..."] [EVOLVE_ROOT=<path-to-evolve/>] cycle-start.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EVOLVE_ROOT="${EVOLVE_ROOT:-$(cd "$SCRIPT_DIR/.." && pwd)}"
REPO_ROOT="$(cd "$EVOLVE_ROOT/.." && pwd)"

# --- Determine next cycle number ---
CYCLES_DIR="$EVOLVE_ROOT/cycles"
mkdir -p "$CYCLES_DIR"
LAST_CYCLE=$(ls "$CYCLES_DIR" 2>/dev/null | grep -E '^[0-9]{4}$' | sort -n | tail -1 || echo "0000")
CYCLE_NUM=$(printf "%04d" $((10#$LAST_CYCLE + 1)))

CYCLE_DIR="$CYCLES_DIR/$CYCLE_NUM"
LEDGER="$EVOLVE_ROOT/ledger/cycles.jsonl"
WORKER_ID="${WORKER_ID:-local}"

# Hypothesis can be passed as env var or first arg
HYPOTHESIS="${1:-${HYPOTHESIS:-"Swap the eviction policy to FIFO as a baseline sanity check."}}"

if [ -d "$CYCLE_DIR" ]; then
    echo "error: cycle $CYCLE_NUM already exists at $CYCLE_DIR" >&2
    exit 1
fi

# --- scaffold cycle directory ---
mkdir -p "$CYCLE_DIR/variants/v001/impl" "$CYCLE_DIR/variants/v001/eval"

cat > "$CYCLE_DIR/cohort.md" <<EOF
# Cycle $CYCLE_NUM Cohort

**Stub run — walking skeleton.**

## Hypothesis H001

$HYPOTHESIS
EOF

# stub impl placeholder
touch "$CYCLE_DIR/variants/v001/impl/.gitkeep"

# stub eval output
cat > "$CYCLE_DIR/variants/v001/eval/fitness.json" <<EOF
{
  "worker_id": "$WORKER_ID",
  "variant": "v001",
  "hit_rate": 0.42,
  "per_op_ns": 120,
  "metadata_bytes": 4096
}
EOF

cat > "$CYCLE_DIR/variants/v001/insight.md" <<'EOF'
# Variant v001 Insight

**Stub — walking skeleton.**

Fitness: hit_rate=0.42, per_op_ns=120, metadata_bytes=4096.
All gates passed (stub). No real evaluation performed.
EOF

cat > "$CYCLE_DIR/cycle-insight.md" <<EOF
# Cycle $CYCLE_NUM Insight

**Stub — walking skeleton.**

One stub variant evaluated. Framework end-to-end path verified.
EOF

cat > "$CYCLE_DIR/winners.json" <<EOF
{
  "cycle": "$CYCLE_NUM",
  "worker_id": "$WORKER_ID",
  "pareto_front": ["v001"]
}
EOF

# --- seal ---
bash "$SCRIPT_DIR/seal.sh" "$CYCLE_DIR"

# --- append ledger row (repo-relative cycle_dir) ---
CYCLE_DIR_REL="evolve/cycles/$CYCLE_NUM"
ROW=$(printf '{"cycle":"%s","worker_id":"%s","cycle_dir":"%s","pareto_front":["v001"]}' \
    "$CYCLE_NUM" "$WORKER_ID" "$CYCLE_DIR_REL")
echo "$ROW" | python3 "$SCRIPT_DIR/ledger-append.py" "$LEDGER"

echo "cycle $CYCLE_NUM complete — sealed at $CYCLE_DIR"
