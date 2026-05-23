#!/usr/bin/env python3
"""Reset evolve/outputs/ to its pristine, just-cloned state."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def main() -> int:
    evolve_root = Path(__file__).resolve().parent.parent
    outputs = evolve_root / "outputs"
    if not outputs.is_dir():
        print(f"error: {outputs} does not exist", file=sys.stderr)
        return 1

    # Delegate to git so ignore rules aren't re-implemented here.
    cmd = ["git", "clean", "-fdx", "--", str(outputs)]
    result = subprocess.run(cmd, cwd=evolve_root.parent, capture_output=True, text=True)
    sys.stdout.write(result.stdout)
    sys.stderr.write(result.stderr)
    if result.returncode == 0 and not result.stdout.strip():
        print("Already in a clean state")
    return result.returncode


if __name__ == "__main__":
    sys.exit(main())
