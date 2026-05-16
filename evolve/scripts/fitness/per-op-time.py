#!/usr/bin/env python3
"""Per-op-time fitness plugin.

Builds and runs the evolve/bench/ microbench harness and emits the plugin
record from `bench.json`. Validates the bench output against the plugin contract.

Environment:
  EVOLVE_ROOT path to the evolve/ directory (default: parent of scripts/)
"""

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

_HERE = Path(__file__).resolve()
_DEFAULT_EVOLVE_ROOT = _HERE.parents[2]
sys.path.insert(0, str(_HERE.parent))

from _common import cargo_build  # noqa: E402
from _contract import ContractError, validate_plugin_dict  # noqa: E402

RUN_TIMEOUT_S = 120
METADATA_TIMEOUT_S = 60


def _bench_dir(evolve_root: Path) -> Path:
    return evolve_root / "bench"


def _target_dir(evolve_root: Path) -> Path:
    """Resolve the bench crate's target directory via `cargo metadata`.

    Works whether `evolve/bench` is a standalone crate or a workspace member —
    in the workspace case the target lives at the workspace root, not under bench/.
    """
    bench_dir = _bench_dir(evolve_root)
    result = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=bench_dir,
        capture_output=True,
        text=True,
        timeout=METADATA_TIMEOUT_S,
    )
    if result.returncode != 0:
        print(f"error: cargo metadata failed:\n{result.stderr}", file=sys.stderr)
        sys.exit(1)
    return Path(json.loads(result.stdout)["target_directory"])


def main():
    evolve_root = Path(os.environ.get("EVOLVE_ROOT", _DEFAULT_EVOLVE_ROOT))
    bench_dir = _bench_dir(evolve_root)
    cargo_build(bench_dir, ["--release"], label="dash-bench")
    binary = _target_dir(evolve_root) / "release" / "dash-bench"

    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as f:
        tmp_out = Path(f.name)
    try:
        try:
            result = subprocess.run(
                [str(binary), "--output", str(tmp_out)],
                capture_output=True,
                text=True,
                timeout=RUN_TIMEOUT_S,
            )
        except subprocess.TimeoutExpired as e:
            print(f"error: dash-bench timed out after {e.timeout}s", file=sys.stderr)
            sys.exit(1)
        if result.returncode != 0:
            print(f"error: dash-bench failed (exit {result.returncode}):\n{result.stderr}", file=sys.stderr)
            sys.exit(1)
        try:
            data = json.loads(tmp_out.read_text())
            validate_plugin_dict(data, source="dash-bench")
        except (json.JSONDecodeError, OSError, ContractError) as e:
            print(f"error: reading/validating bench output: {e}", file=sys.stderr)
            sys.exit(1)
    finally:
        tmp_out.unlink(missing_ok=True)

    print(json.dumps(data))


if __name__ == "__main__":
    main()
