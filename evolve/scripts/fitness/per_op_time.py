"""Per-op-time fitness plugin.

Builds and runs the evolve/bench/ microbench harness and returns the plugin
record from `bench.json`. Validates the bench output against the plugin contract.
"""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

_HERE = Path(__file__).resolve()
sys.path.insert(0, str(_HERE.parent))

from _common import cargo_build, cargo_env  # noqa: E402
from _contract import validate_plugin_dict  # noqa: E402

RUN_TIMEOUT_S = 120
METADATA_TIMEOUT_S = 60


def _resolve_target_dir(bench_dir: Path, target_dir: Path | None) -> Path:
    """Resolve the bench crate's target directory via `cargo metadata`.

    Works whether `evolve/bench` is a standalone crate or a workspace member —
    in the workspace case the target lives at the workspace root, not under bench/.
    Respects CARGO_TARGET_DIR when `target_dir` is supplied.
    """

    result = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=bench_dir,
        capture_output=True,
        text=True,
        timeout=METADATA_TIMEOUT_S,
        check=True,
        env=cargo_env(target_dir),
    )
    return Path(json.loads(result.stdout)["target_directory"])


def compute(*, evolve_root: Path, target_dir: Path | None = None) -> dict:
    """Build + run dash-bench; return the validated plugin record."""

    bench_dir = evolve_root / "bench"
    cargo_build(bench_dir, ["--release"], label="dash-bench", target_dir=target_dir)
    binary = _resolve_target_dir(bench_dir, target_dir) / "release" / "dash-bench"

    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as f:
        tmp_out = Path(f.name)
    try:
        subprocess.run(
            [str(binary), "--output", str(tmp_out)],
            capture_output=True,
            text=True,
            timeout=RUN_TIMEOUT_S,
            check=True,
        )
        data = json.loads(tmp_out.read_text())
        validate_plugin_dict(data, source="dash-bench")
    finally:
        tmp_out.unlink(missing_ok=True)

    return data
