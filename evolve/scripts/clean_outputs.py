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
    return subprocess.run(cmd, cwd=evolve_root.parent).returncode


if __name__ == "__main__":
    sys.exit(main())
