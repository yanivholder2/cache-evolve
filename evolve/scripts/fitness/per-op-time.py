#!/usr/bin/env python3
"""Per-op-time fitness plugin.

Builds and runs the evolve/bench/ microbench harness and emits:
  { "dim": "per-op-time", "values": [...], "summary": float, "higher_is_better": false }

Usage: per-op-time.py
Environment:
  EVOLVE_ROOT path to the evolve/ directory (default: parent of scripts/)
  REPO_ROOT   path to the dash-cache-rs repo (used to locate evolve/)
"""

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

_HERE = Path(__file__).resolve()
_DEFAULT_EVOLVE_ROOT = _HERE.parents[2]

BENCH_PROFILES = ("release", "debug")


def _bench_dir(evolve_root: Path) -> Path:
    return evolve_root / "bench"


def _find_binary(evolve_root: Path) -> Path | None:
    bench_dir = _bench_dir(evolve_root)
    for profile in BENCH_PROFILES:
        p = bench_dir / "target" / profile / "dash-bench"
        if p.exists():
            return p
    return None


def _build_binary(evolve_root: Path) -> Path:
    bench_dir = _bench_dir(evolve_root)
    result = subprocess.run(
        ["cargo", "build", "--release"],
        cwd=bench_dir,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print(f"error: cargo build for bench failed:\n{result.stderr}", file=sys.stderr)
        sys.exit(1)
    return bench_dir / "target" / "release" / "dash-bench"


def main():
    evolve_root = Path(os.environ.get("EVOLVE_ROOT", _DEFAULT_EVOLVE_ROOT))

    binary = _find_binary(evolve_root) or _build_binary(evolve_root)

    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as f:
        tmp_out = Path(f.name)

    result = subprocess.run(
        [str(binary), "--output", str(tmp_out)],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print(f"error: dash-bench failed: {result.stderr}", file=sys.stderr)
        sys.exit(1)

    try:
        data = json.loads(tmp_out.read_text())
    except (json.JSONDecodeError, OSError) as e:
        print(f"error: reading bench output: {e}", file=sys.stderr)
        sys.exit(1)
    finally:
        tmp_out.unlink(missing_ok=True)

    print(json.dumps(data))


if __name__ == "__main__":
    main()
