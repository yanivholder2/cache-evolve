#!/usr/bin/env bash
# Default real-LLM adapter for the agent-agnostic LLM CLI contract (ADR-0005).
#
# Contract:
#   $EVOLVE_LLM_CMD <agent-name> <output-dir> [--writable <path>...]
#
# The agent body (`.agents/agents/<agent-name>.md`) and the caller-written
# `<output-dir>/task.md` are concatenated and fed to `gemini -p`. The agent is
# expected to use the underlying CLI's file tools to Read context and Write
# artifacts under <output-dir>.
#
# Tool permissioning (write-scope enforcement) is currently prompt-level only:
# each agent's `.md` body states "only write under $OUTPUT_DIR". A future
# hardening pass will plumb Gemini-policy allowlists through `--writable`
# here; ADR-0005 documents the gap.
set -euo pipefail

if [ "$#" -lt 2 ]; then
    echo "usage: $0 <agent-name> <output-dir> [--writable <path>...]" >&2
    exit 2
fi

AGENT_NAME="$1"
OUTPUT_DIR="$2"
shift 2

# Drain --writable <path> pairs. Currently advisory (prompt-level only); kept
# in argv for forward compatibility once Gemini policy syntax is firm.
WRITABLE=()
while [ "$#" -gt 0 ]; do
    case "$1" in
        --writable)
            shift
            [ "$#" -ge 1 ] || { echo "$0: --writable needs a path" >&2; exit 2; }
            WRITABLE+=("$1")
            shift
            ;;
        *)
            echo "$0: unknown arg: $1" >&2
            exit 2
            ;;
    esac
done

REPO_ROOT="${REPO_ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
AGENT_FILE="${REPO_ROOT}/.agents/agents/${AGENT_NAME}.md"
TASK_FILE="${OUTPUT_DIR}/task.md"

[ -f "$AGENT_FILE" ] || { echo "$0: agent prompt not found: $AGENT_FILE" >&2; exit 1; }
[ -f "$TASK_FILE" ] || { echo "$0: task.md not found: $TASK_FILE" >&2; exit 1; }

# Concatenate agent body + task. Both are markdown; the agent body is the
# "system prompt" and task.md is the per-invocation assignment.
PROMPT="$(cat "$AGENT_FILE")"$'\n\n---\n\n'"$(cat "$TASK_FILE")"

export OUTPUT_DIR
exec gemini -p "$PROMPT"
