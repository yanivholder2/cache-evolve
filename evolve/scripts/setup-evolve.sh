#!/usr/bin/env bash
# Set up the evolve Python environment.
# Creates a venv at evolve/.venv and installs requirements.txt into it.
#
# Usage: bash evolve/scripts/setup-evolve.sh
# Run once before using pytest or any evolve scripts that need Python deps.
set -euo pipefail

EVOLVE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VENV="$EVOLVE_DIR/.venv"

if [ ! -d "$VENV" ]; then
    python3 -m venv "$VENV"
fi

"$VENV/bin/pip" install -q -r "$EVOLVE_DIR/requirements.txt"

echo "evolve env ready"
echo "run tests with: $VENV/bin/pytest $EVOLVE_DIR/tests/"
