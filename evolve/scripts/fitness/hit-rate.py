#!/usr/bin/env python3
"""Hit-rate fitness plugin.

Runs the evolve-hit-rate binary against a set of configurations and emits:
  { "dim": "hit-rate", "values": [...], "summary": float, "higher_is_better": true }

Usage: hit-rate.py
Environment:
  REPO_ROOT   path to the dash-cache-rs repo (default: 4 levels up from this file)
  EVOLVE_ROOT path to the evolve/ directory (default: parent of scripts/)
"""

import json
import os
import subprocess
import sys
from pathlib import Path

_HERE = Path(__file__).resolve()
_DEFAULT_REPO_ROOT = _HERE.parents[3]
_DEFAULT_EVOLVE_ROOT = _HERE.parents[1]

CONFIGS = [
    {"cache": "assoc", "size": 16},
    {"cache": "assoc", "size": 32},
    {"cache": "dash", "size": 16},
    {"cache": "dash", "size": 32},
]
EVENTS = 2000
SEED = 42


def _find_binary(repo_root: Path) -> Path | None:
    for profile in ("release", "debug"):
        p = repo_root / "target" / profile / "evolve-hit-rate"
        if p.exists():
            return p
    return None


def _build_binary(repo_root: Path) -> Path:
    result = subprocess.run(
        ["cargo", "build", "--bin", "evolve-hit-rate"],
        cwd=repo_root,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print(f"error: cargo build failed:\n{result.stderr}", file=sys.stderr)
        sys.exit(1)
    return repo_root / "target" / "debug" / "evolve-hit-rate"


def main():
    repo_root = Path(os.environ.get("REPO_ROOT", _DEFAULT_REPO_ROOT))

    binary = _find_binary(repo_root) or _build_binary(repo_root)

    values = []
    for cfg in CONFIGS:
        result = subprocess.run(
            [str(binary), "--cache", cfg["cache"], "--size", str(cfg["size"]),
             "--events", str(EVENTS), "--seed", str(SEED)],
            capture_output=True,
            text=True,
        )
        if result.returncode != 0:
            print(f"error: evolve-hit-rate failed: {result.stderr}", file=sys.stderr)
            sys.exit(1)

        data = json.loads(result.stdout)
        for v in data["values"]:
            values.append({
                "cache": cfg["cache"],
                "size": cfg["size"],
                "hit_rate": v["hit_rate"],
            })

    if not values:
        print("error: no hit-rate values collected", file=sys.stderr)
        sys.exit(1)

    summary = sum(v["hit_rate"] for v in values) / len(values)
    out = {
        "dim": "hit-rate",
        "values": values,
        "summary": round(summary, 6),
        "higher_is_better": True,
    }
    print(json.dumps(out))


if __name__ == "__main__":
    main()
