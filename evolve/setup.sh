#!/usr/bin/env bash
set -euo pipefail

EVOLVE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VENV="$EVOLVE_DIR/.venv"

if [ ! -d "$VENV" ]; then
    python3 -m venv "$VENV"
fi

"$VENV/bin/pip" install -q -r "$EVOLVE_DIR/requirements.txt"

echo "evolve env ready — activate with: source $VENV/bin/activate"
echo "or run tests with: $VENV/bin/pytest evolve/tests/"
