#!/usr/bin/env bash
# Cycle orchestrator — runs G1+G2 gates and real fitness plugins, aggregates, computes Pareto front.
#
# Usage: [HYPOTHESIS="..."] [EVOLVE_ROOT=<path-to-evolve/>] [REPO_ROOT=<dash-crate-root>] [WORKER_ID=<id>] cycle-start.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EVOLVE_ROOT="${EVOLVE_ROOT:-$(cd "$SCRIPT_DIR/.." && pwd)}"
# Allow callers to pin REPO_ROOT (e.g. tests that copy evolve/ to a temp dir)
REPO_ROOT="${REPO_ROOT:-$(cd "$EVOLVE_ROOT/.." && pwd)}"
export REPO_ROOT EVOLVE_ROOT

# --- Determine next cycle number ---
CYCLES_DIR="$EVOLVE_ROOT/cycles"
mkdir -p "$CYCLES_DIR"
LAST_CYCLE=$(ls "$CYCLES_DIR" 2>/dev/null | grep -E '^[0-9]{4}$' | sort -n | tail -1 || echo "0000")
CYCLE_NUM=$(printf "%04d" $((10#$LAST_CYCLE + 1)))

CYCLE_DIR="$CYCLES_DIR/$CYCLE_NUM"
LEDGER="$EVOLVE_ROOT/ledger/cycles.jsonl"
WORKER_ID="${WORKER_ID:-local}"
export WORKER_ID

HYPOTHESIS="${1:-${HYPOTHESIS:-"Swap the eviction policy to FIFO as a baseline sanity check."}}"

if [ -d "$CYCLE_DIR" ]; then
    echo "error: cycle $CYCLE_NUM already exists at $CYCLE_DIR" >&2
    exit 1
fi

FIXTURES_DIR="$EVOLVE_ROOT/fixtures"

# --- Python interpreter from venv if available ---
PYTHON="${EVOLVE_ROOT}/.venv/bin/python3"
if [ ! -x "$PYTHON" ]; then
    PYTHON="python3"
fi

# --- scaffold cycle directory ---
mkdir -p "$CYCLE_DIR/variants/v001/impl" "$CYCLE_DIR/variants/v001/eval/fitness"

# --- populate stub variant impl from good fixture ---
cp "$FIXTURES_DIR/good/impl/segment.rs"   "$CYCLE_DIR/variants/v001/impl/segment.rs"
cp "$FIXTURES_DIR/good/impl/manifest.toml" "$CYCLE_DIR/variants/v001/impl/manifest.toml"

# --- run gate-check (G1 + G2; G3 is still a stub) ---
bash "$SCRIPT_DIR/gate-check.sh" "$CYCLE_DIR/variants/v001/impl" "$CYCLE_NUM" "v001" || {
    echo "error: gate-check failed for v001 in cycle $CYCLE_NUM" >&2
    exit 1
}

cat > "$CYCLE_DIR/cohort.md" <<EOF
# Cycle $CYCLE_NUM Cohort

## Hypothesis H001

$HYPOTHESIS
EOF

# --- run fitness plugins for v001 ---
EVAL_DIR="$CYCLE_DIR/variants/v001/eval"
FITNESS_DIR="$EVAL_DIR/fitness"

echo "running fitness plugins for v001..."

for plugin in hit-rate per-op-time metadata-size; do
    echo "  $plugin..."
    OUT="$FITNESS_DIR/${plugin}.json"
    "$PYTHON" "$SCRIPT_DIR/fitness/${plugin}.py" > "$OUT"
    if [ ! -s "$OUT" ]; then
        echo "error: plugin $plugin produced empty output at $OUT" >&2
        exit 1
    fi
    "$PYTHON" -c "import json,sys; json.load(open(sys.argv[1]))" "$OUT" || {
        echo "error: plugin $plugin produced invalid JSON at $OUT" >&2
        exit 1
    }
done

# --- aggregate fitness vector ---
"$PYTHON" "$SCRIPT_DIR/fitness-aggregate.py" v001 "$FITNESS_DIR" > "$EVAL_DIR/fitness-vector.json"

# --- build variant insight from fitness data ---
FV_JSON=$(cat "$EVAL_DIR/fitness-vector.json")
DIM_SUMMARY=$("$PYTHON" "$SCRIPT_DIR/dim-summary.py" "$EVAL_DIR/fitness-vector.json")

cat > "$CYCLE_DIR/variants/v001/insight.md" <<EOF
# Variant v001 Insight

Fitness: $DIM_SUMMARY
EOF

# --- compute Pareto front (single variant -> trivially itself) ---
# TODO: when real cohort generation lands, build FV_ARRAY from all variants' fitness-vector.json files.
FV_ARRAY="[$FV_JSON]"
PARETO_FRONT=$(echo "$FV_ARRAY" | "$PYTHON" "$SCRIPT_DIR/pareto.py" -)
VARIANT_COUNT=$(echo "$FV_ARRAY" | "$PYTHON" -c "import json,sys; print(len(json.load(sys.stdin)))")

cat > "$CYCLE_DIR/winners.json" <<EOF
{
  "cycle": "$CYCLE_NUM",
  "worker_id": "$WORKER_ID",
  "pareto_front": $PARETO_FRONT,
  "fitness_vectors": [$FV_JSON]
}
EOF

# --- run held-out evaluation on Pareto front (only Pareto winners; never all variants) ---
echo "running held-out fitness for Pareto front..."
"$PYTHON" "$SCRIPT_DIR/holdout-runner.py" "$CYCLE_DIR"
HOLDOUT_REGRESSION=$("$PYTHON" -c "import json,sys; print(str(json.load(open(sys.argv[1]))['any_regression']).lower())" "$CYCLE_DIR/holdout.json")

cat > "$CYCLE_DIR/cycle-insight.md" <<EOF
# Cycle $CYCLE_NUM Insight

Pareto front: $PARETO_FRONT

Evaluated $VARIANT_COUNT variant(s) with real fitness plugins (hit-rate, per-op-time, metadata-size).

Held-out regression: $HOLDOUT_REGRESSION (see holdout.json for per-variant deltas).
EOF

# --- seal ---
bash "$SCRIPT_DIR/seal.sh" "$CYCLE_DIR"

# --- append ledger row (repo-relative cycle_dir) ---
CYCLE_DIR_REL="evolve/cycles/$CYCLE_NUM"
ROW=$(printf '{"cycle":"%s","worker_id":"%s","cycle_dir":"%s","pareto_front":%s,"holdout_regression":%s}' \
    "$CYCLE_NUM" "$WORKER_ID" "$CYCLE_DIR_REL" "$PARETO_FRONT" "$HOLDOUT_REGRESSION")
echo "$ROW" | "$PYTHON" "$SCRIPT_DIR/ledger-append.py" "$LEDGER"

# --- update leaderboard (after ledger so this cycle appears) ---
"$PYTHON" "$SCRIPT_DIR/leaderboard.py" \
    --evolve-root "$EVOLVE_ROOT" \
    --output "$EVOLVE_ROOT/leaderboard.md"

echo "cycle $CYCLE_NUM complete — sealed at $CYCLE_DIR"
