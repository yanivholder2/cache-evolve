#!/usr/bin/env python3
"""Reset evolve/outputs/ to its pristine, just-cloned state by deleting every
untracked file and directory under it (both gitignored and merely untracked).
Tracked files (the per-subdir .gitkeep markers) are preserved.

Wraps `git clean -fdx evolve/outputs` so the ignore rules are sourced from
git itself rather than re-implemented here.
"""

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

    cmd = ["git", "clean", "-fdx", "--", str(outputs)]
    return subprocess.run(cmd, cwd=evolve_root.parent).returncode


if __name__ == "__main__":
    sys.exit(main())
