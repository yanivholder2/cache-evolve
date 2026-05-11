#!/bin/bash
# Run the script passed as $1 only in Claude Code web sessions; no-op on local CLI.
# Detection: $CLAUDE_CODE_REMOTE == "true" in remote sessions, unset otherwise.
set -euo pipefail

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

if [ $# -lt 1 ]; then
  echo "web-only.sh: missing script argument" >&2
  exit 2
fi

# Tolerate not-yet-implemented hook scripts: skip cleanly instead of failing the
# session. This lets settings.json reference scripts before they land.
target="$1"
if [ ! -f "$target" ]; then
  echo "web-only.sh: target $target not found, skipping" >&2
  exit 0
fi

exec "$@"
