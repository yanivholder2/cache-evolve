#!/usr/bin/env python3
"""Metadata-size fitness plugin.

Runs the evolve-metadata-size binary and emits:
  { "dim": "metadata-size", "values": [...], "summary": float, "higher_is_better": false }

Usage: metadata-size.py
Environment:
  REPO_ROOT   path to the dash-cache-rs repo (default: 4 levels up from this file)
"""

import json
import os
import subprocess
import sys
from pathlib import Path

_HERE = Path(__file__).resolve()
_DEFAULT_REPO_ROOT = _HERE.parents[3]


def _find_binary(repo_root: Path) -> Path | None:
    for profile in ("release", "debug"):
        p = repo_root / "target" / profile / "evolve-metadata-size"
        if p.exists():
            return p
    return None


def _build_binary(repo_root: Path) -> Path:
    result = subprocess.run(
        ["cargo", "build", "--bin", "evolve-metadata-size"],
        cwd=repo_root,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print(f"error: cargo build failed:\n{result.stderr}", file=sys.stderr)
        sys.exit(1)
    return repo_root / "target" / "debug" / "evolve-metadata-size"


def main():
    repo_root = Path(os.environ.get("REPO_ROOT", _DEFAULT_REPO_ROOT))

    binary = _find_binary(repo_root) or _build_binary(repo_root)

    result = subprocess.run(
        [str(binary)],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print(f"error: evolve-metadata-size failed: {result.stderr}", file=sys.stderr)
        sys.exit(1)

    data = json.loads(result.stdout)
    print(json.dumps(data))


if __name__ == "__main__":
    main()
