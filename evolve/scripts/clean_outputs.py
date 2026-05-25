#!/usr/bin/env python3
"""Reset evolve/outputs/ to its pristine, just-cloned state."""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


def _restore_write_bits(root: Path) -> None:
    # Sealed cycles strip write bits (see src/cycle/seal.py); restore them so
    # git clean can remove entries from sealed directories.
    for dirpath, dirnames, filenames in os.walk(root):
        for name in filenames + dirnames:
            p = Path(dirpath) / name
            os.chmod(p, p.stat().st_mode | 0o200)
    os.chmod(root, root.stat().st_mode | 0o200)


def main() -> int:
    evolve_root = Path(__file__).resolve().parent.parent
    outputs = evolve_root / "outputs"
    if not outputs.is_dir():
        print(f"error: {outputs} does not exist", file=sys.stderr)
        return 1

    _restore_write_bits(outputs)

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
