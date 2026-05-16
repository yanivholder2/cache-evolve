#!/usr/bin/env bash
# Cycle orchestrator — fans out a cohort of N Variants, runs real G1+G2 gates and
# fitness plugins per variant, aggregates a Pareto front, seals the cycle, and
# appends one ledger row.
#
# Usage:
#   [HYPOTHESES="h1\nh2\n..."] [COHORT_SIZE=N] [VARIANT_IMPLEMENTER=<cmd>] \
#   [EVOLVE_ROOT=<path>] [REPO_ROOT=<dash-crate-root>] [WORKER_ID=<id>] \
#   cycle-start.sh
#
# HYPOTHESES overrides COHORT_SIZE (one Variant per line).
# COHORT_SIZE overrides the config.toml default (6).
# VARIANT_IMPLEMENTER is invoked once per Variant in parallel as
#   <cmd> <hypothesis-file> <impl-out-dir>
# and defaults to scripts/variant-implementer.sh (copies the good fixture).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EVOLVE_ROOT="${EVOLVE_ROOT:-$(cd "$SCRIPT_DIR/.." && pwd)}"
REPO_ROOT="${REPO_ROOT:-$(cd "$EVOLVE_ROOT/.." && pwd)}"
export REPO_ROOT EVOLVE_ROOT

WORKER_ID="${WORKER_ID:-local}"
export WORKER_ID

VARIANT_IMPLEMENTER="${VARIANT_IMPLEMENTER:-$SCRIPT_DIR/variant-implementer.sh}"

# --- Resolve cohort size: env override > config.toml > built-in default ---
CONFIG_FILE="$EVOLVE_ROOT/config.toml"
if [ -z "${COHORT_SIZE:-}" ]; then
    if [ -f "$CONFIG_FILE" ]; then
        COHORT_SIZE=$(grep -E '^[[:space:]]*cohort_size[[:space:]]*=' "$CONFIG_FILE" \
            | head -1 \
            | sed -E 's/.*=[[:space:]]*([0-9]+).*/\1/')
    fi
fi
COHORT_SIZE="${COHORT_SIZE:-6}"

if ! [[ "$COHORT_SIZE" =~ ^[0-9]+$ ]] || [ "$COHORT_SIZE" -lt 1 ]; then
    echo "error: COHORT_SIZE must be a positive integer, got: $COHORT_SIZE" >&2
    exit 1
fi

# --- Resolve hypotheses list ---
HYPS=()
if [ -n "${HYPOTHESES:-}" ]; then
    # newline-separated; respect $'\n' from the caller
    while IFS= read -r line; do
        [ -n "$line" ] && HYPS+=("$line")
    done <<< "$HYPOTHESES"
    COHORT_SIZE=${#HYPS[@]}
else
    for i in $(seq 1 "$COHORT_SIZE"); do
        HYPS+=("Default hypothesis H$(printf '%02d' "$i"): exploratory Variant from the walking-skeleton stub.")
    done
fi

# --- Determine next cycle number ---
CYCLES_DIR="$EVOLVE_ROOT/cycles"
mkdir -p "$CYCLES_DIR"
LAST_CYCLE=$(ls "$CYCLES_DIR" 2>/dev/null | grep -E '^[0-9]{4}$' | sort -n | tail -1 || echo "0000")
CYCLE_NUM=$(printf "%04d" $((10#$LAST_CYCLE + 1)))

CYCLE_DIR="$CYCLES_DIR/$CYCLE_NUM"
LEDGER="$EVOLVE_ROOT/ledger/cycles.jsonl"

if [ -d "$CYCLE_DIR" ]; then
    echo "error: cycle $CYCLE_NUM already exists at $CYCLE_DIR" >&2
    exit 1
fi

# --- Python interpreter ---
PYTHON="${EVOLVE_ROOT}/.venv/bin/python3"
if [ ! -x "$PYTHON" ]; then
    PYTHON="python3"
fi

# --- Scaffold cycle directory + hypotheses ---
mkdir -p "$CYCLE_DIR/hypotheses"

{
    echo "# Cycle $CYCLE_NUM Cohort"
    echo
    echo "Cohort size: $COHORT_SIZE"
    echo
    echo "## Hypotheses"
    echo
    for i in $(seq 1 "$COHORT_SIZE"); do
        HID=$(printf "H%02d" "$i")
        echo "- [\`$HID\`](hypotheses/$HID.md)"
    done
} > "$CYCLE_DIR/cohort.md"

for i in $(seq 1 "$COHORT_SIZE"); do
    HID=$(printf "H%02d" "$i")
    idx=$((i - 1))
    {
        echo "# Hypothesis $HID"
        echo
        echo "${HYPS[$idx]}"
    } > "$CYCLE_DIR/hypotheses/$HID.md"
done

# --- Fan-out: invoke variant-implementer in parallel (P1) ---
PIDS=()
for i in $(seq 1 "$COHORT_SIZE"); do
    HID=$(printf "H%02d" "$i")
    VID=$(printf "v%03d" "$i")
    VARIANT_DIR="$CYCLE_DIR/variants/$VID"
    mkdir -p "$VARIANT_DIR/impl" "$VARIANT_DIR/eval/fitness"
    (
        set +e
        "$VARIANT_IMPLEMENTER" \
            "$CYCLE_DIR/hypotheses/$HID.md" \
            "$VARIANT_DIR/impl" \
            > "$VARIANT_DIR/implementer.log" 2>&1
        echo "$?" > "$VARIANT_DIR/implementer.exit"
    ) &
    PIDS+=("$!")
done
for pid in "${PIDS[@]}"; do
    wait "$pid" || true
done

for i in $(seq 1 "$COHORT_SIZE"); do
    VID=$(printf "v%03d" "$i")
    VARIANT_DIR="$CYCLE_DIR/variants/$VID"
    EXIT=$(cat "$VARIANT_DIR/implementer.exit")
    if [ "$EXIT" != "0" ]; then
        echo "error: variant-implementer for $VID failed (exit $EXIT):" >&2
        cat "$VARIANT_DIR/implementer.log" >&2
        exit 1
    fi
done

# --- Per-variant gates + fitness ---
# Gates run sequentially (cargo serializes against its target dir); fitness
# plugins are deterministic short Python runs.
for i in $(seq 1 "$COHORT_SIZE"); do
    VID=$(printf "v%03d" "$i")
    VARIANT_DIR="$CYCLE_DIR/variants/$VID"
    IMPL_DIR="$VARIANT_DIR/impl"

    bash "$SCRIPT_DIR/gate-check.sh" "$IMPL_DIR" "$CYCLE_NUM" "$VID" || {
        echo "error: gate-check failed for $VID in cycle $CYCLE_NUM" >&2
        exit 1
    }

    EVAL_DIR="$VARIANT_DIR/eval"
    FITNESS_DIR="$EVAL_DIR/fitness"

    for plugin in hit-rate per-op-time metadata-size; do
        OUT="$FITNESS_DIR/${plugin}.json"
        "$PYTHON" "$SCRIPT_DIR/fitness/${plugin}.py" > "$OUT"
        if [ ! -s "$OUT" ]; then
            echo "error: plugin $plugin produced empty output for $VID at $OUT" >&2
            exit 1
        fi
        "$PYTHON" -c "import json,sys; json.load(open(sys.argv[1]))" "$OUT" || {
            echo "error: plugin $plugin produced invalid JSON for $VID at $OUT" >&2
            exit 1
        }
    done

    "$PYTHON" "$SCRIPT_DIR/fitness-aggregate.py" "$VID" "$FITNESS_DIR" > "$EVAL_DIR/fitness-vector.json"
done

# --- Collect fitness vectors and compute Pareto front over the cohort ---
FV_LIST=()
for i in $(seq 1 "$COHORT_SIZE"); do
    VID=$(printf "v%03d" "$i")
    FV_LIST+=("$(cat "$CYCLE_DIR/variants/$VID/eval/fitness-vector.json")")
done
FV_ARRAY="[$(IFS=,; echo "${FV_LIST[*]}")]"
PARETO_FRONT=$(echo "$FV_ARRAY" | "$PYTHON" "$SCRIPT_DIR/pareto.py" -)

# --- Per-variant insights ---
for i in $(seq 1 "$COHORT_SIZE"); do
    VID=$(printf "v%03d" "$i")
    EVAL_DIR="$CYCLE_DIR/variants/$VID/eval"
    DIM_SUMMARY=$("$PYTHON" "$SCRIPT_DIR/dim-summary.py" "$EVAL_DIR/fitness-vector.json")
    cat > "$CYCLE_DIR/variants/$VID/insight.md" <<EOF
# Variant $VID Insight

Fitness: $DIM_SUMMARY
EOF
done

cat > "$CYCLE_DIR/winners.json" <<EOF
{
  "cycle": "$CYCLE_NUM",
  "worker_id": "$WORKER_ID",
  "pareto_front": $PARETO_FRONT,
  "fitness_vectors": $FV_ARRAY
}
EOF

cat > "$CYCLE_DIR/cycle-insight.md" <<EOF
# Cycle $CYCLE_NUM Insight

Cohort size: $COHORT_SIZE
Pareto front: $PARETO_FRONT

Evaluated $COHORT_SIZE variant(s) with real fitness plugins (hit-rate, per-op-time, metadata-size).
EOF

# --- Seal ---
bash "$SCRIPT_DIR/seal.sh" "$CYCLE_DIR"

# --- Append ledger row (repo-relative cycle_dir) ---
CYCLE_DIR_REL="evolve/cycles/$CYCLE_NUM"
ROW=$(printf '{"cycle":"%s","worker_id":"%s","cycle_dir":"%s","cohort_size":%s,"pareto_front":%s}' \
    "$CYCLE_NUM" "$WORKER_ID" "$CYCLE_DIR_REL" "$COHORT_SIZE" "$PARETO_FRONT")
echo "$ROW" | "$PYTHON" "$SCRIPT_DIR/ledger-append.py" "$LEDGER"

# --- Regenerate leaderboard ---
"$PYTHON" "$SCRIPT_DIR/leaderboard.py" \
    --evolve-root "$EVOLVE_ROOT" \
    --output "$EVOLVE_ROOT/leaderboard.md"

echo "cycle $CYCLE_NUM complete — sealed at $CYCLE_DIR (cohort=$COHORT_SIZE)"
