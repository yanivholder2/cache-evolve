#!/usr/bin/env bash
# Deterministic LLM stub for EVOLVE_LLM_CMD in tests.
#
# Honours these env vars (set per-test):
#   STUB_OUT      — exact text to print on stdout (newline-trimmed)
#   STUB_LINES    — emit N lines "stub hypothesis K" (K=1..N)
#   STUB_EXIT     — exit code (default 0)
#   STUB_CAPTURE  — file to append the prompt argv to
#
# The prompt is the final positional argument (per ADR-0005).
set -uo pipefail

PROMPT="${!#-}"

if [ -n "${STUB_CAPTURE:-}" ]; then
    {
        echo "----- invocation -----"
        echo "$PROMPT"
    } >> "$STUB_CAPTURE"
fi

if [ -n "${STUB_OUT:-}" ]; then
    printf '%s' "$STUB_OUT"
elif [ -n "${STUB_LINES:-}" ]; then
    for ((i=1; i<=STUB_LINES; i++)); do
        echo "stub hypothesis $i"
    done
fi

exit "${STUB_EXIT:-0}"
