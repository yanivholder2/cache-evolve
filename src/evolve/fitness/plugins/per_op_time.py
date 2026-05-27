"""Per-op-time fitness plugin."""

from __future__ import annotations

import json
import subprocess
import tempfile
from pathlib import Path

from evolve._io import read_utf8
from evolve.fitness._contract import validate_plugin_dict
from evolve.fitness.plugins._common import cargo_build, cargo_env
from evolve.paths import SourcesLayout

RUN_TIMEOUT_S = 120
METADATA_TIMEOUT_S = 60


def _resolve_target_dir(bench_dir: Path, target_dir: Path | None) -> Path:
    """Returns the bench crate's target directory.

    Queries cargo because a workspace member's target lives at the workspace root, not under bench/.
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


def compute(*, repo_root: Path, target_dir: Path | None = None) -> dict:
    """Runs dash-bench and returns the validated plugin record."""

    bench_dir = SourcesLayout(root=repo_root).bench_dir
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
        data = json.loads(read_utf8(tmp_out))
        validate_plugin_dict(data, source="dash-bench")
    finally:
        tmp_out.unlink(missing_ok=True)

    return data
