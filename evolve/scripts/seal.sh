#!/usr/bin/env bash
# Mark a cycle directory immutable by dropping write bits and writing a SEALED marker.
# Convention-enforced: the framework never writes into a sealed cycle; chmod is a safety net.
#
# Usage: seal.sh <cycle-dir>
set -euo pipefail

if [ $# -ne 1 ]; then
    echo "usage: seal.sh <cycle-dir>" >&2
    exit 1
fi

CYCLE_DIR="$1"

if [ ! -d "$CYCLE_DIR" ]; then
    echo "error: not a directory: $CYCLE_DIR" >&2
    exit 1
fi

if [ -f "$CYCLE_DIR/SEALED" ]; then
    echo "already sealed: $CYCLE_DIR" >&2
    exit 0
fi

# Write the marker before removing write bits so we can create it.
echo "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > "$CYCLE_DIR/SEALED"
chmod -R a-w "$CYCLE_DIR"
echo "sealed: $CYCLE_DIR"
