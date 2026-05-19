#!/usr/bin/env bash
# Deterministic LLM adapter stub for the agent-agnostic contract (ADR-0005).
#
# Contract:
#   $EVOLVE_LLM_CMD <agent-name> <output-dir> [--writable <path>...]
#
# Dispatches by <agent-name>; writes canned files into <output-dir> instead of
# calling a real model.
#
# Env vars (set per-test):
#   STUB_OUT      — newline-separated hypothesis bodies; one file per non-empty
#                   line, named H01.md..H<NN>.md (hypothesis-writer agent).
#   STUB_LINES    — N: write N stub files "stub hypothesis K" (K=1..N).
#                   Ignored when STUB_OUT is set.
#   STUB_EXIT     — exit code (default 0). Honored after capture, before write.
#   STUB_CAPTURE  — file to append the invocation + task.md body to.
set -uo pipefail

if [ "$#" -lt 2 ]; then
    echo "usage: $0 <agent-name> <output-dir> [--writable <path>...]" >&2
    exit 2
fi

AGENT_NAME="$1"
OUTPUT_DIR="$2"
shift 2

WRITABLE_ARGS=("$@")
TASK_FILE="${OUTPUT_DIR}/task.md"

if [ -n "${STUB_CAPTURE:-}" ]; then
    {
        echo "----- invocation -----"
        echo "agent: $AGENT_NAME"
        echo "output-dir: $OUTPUT_DIR"
        echo "argv-tail: ${WRITABLE_ARGS[*]:-}"
        echo "----- task.md -----"
        [ -f "$TASK_FILE" ] && cat "$TASK_FILE" || echo "(no task.md)"
    } >> "$STUB_CAPTURE"
fi

if [ "${STUB_EXIT:-0}" != "0" ]; then
    exit "${STUB_EXIT}"
fi

case "$AGENT_NAME" in
    hypothesis-writer)
        mkdir -p "$OUTPUT_DIR"
        if [ -n "${STUB_OUT:-}" ]; then
            i=1
            while IFS= read -r line; do
                # Skip blank lines so tests can use trailing newlines safely.
                [ -z "$line" ] && continue
                printf '%s\n' "$line" > "$(printf '%s/H%02d.md' "$OUTPUT_DIR" "$i")"
                i=$((i+1))
            done <<< "${STUB_OUT}"
        elif [ -n "${STUB_LINES:-}" ]; then
            for ((i=1; i<=STUB_LINES; i++)); do
                printf 'stub hypothesis %d\n' "$i" > "$(printf '%s/H%02d.md' "$OUTPUT_DIR" "$i")"
            done
        fi
        ;;
    *)
        echo "$0: unknown agent: $AGENT_NAME" >&2
        exit 1
        ;;
esac

exit 0
