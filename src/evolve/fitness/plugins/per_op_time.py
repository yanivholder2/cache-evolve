"""Per-op-time fitness plugin."""

from __future__ import annotations

import json
import subprocess
import tempfile
from pathlib import Path

from evolve._io import read_utf8
from evolve.fitness._contract import validate_plugin_dict
from evolve.fitness.plugins._common import cargo_build, cargo_env
from evolve.paths import Layout
from evolve.subject import load_subject

RUN_TIMEOUT_S = 120
METADATA_TIMEOUT_S = 60


def _resolve_target_dir(bench_dir: Path, target_dir: Path | None) -> Path:
    """Returns the bench crate's target directory.

    When target_dir is set it is authoritative (cargo_build runs with
    CARGO_TARGET_DIR=target_dir), so return it without spawning cargo. Fall back
    to `cargo metadata` only when target_dir is None, because a workspace
    member's target lives at the workspace root, not under bench/.
    """

    if target_dir is not None:
        return target_dir

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


def compute(*, layout: Layout, target_dir: Path | None = None) -> dict:
    """Runs the selected subject's microbench and returns the validated record.

    The subject must declare a `[bench]` table in its manifest (`runner` only
    schedules this plugin when it does)."""

    subject = load_subject(layout.sources.subject_dir)
    if subject.bench is None:
        raise RuntimeError(f"subject {subject.name!r} declares no [bench]; per-op-time is unavailable")
    binary_name = subject.bench.binary_name

    bench_dir = layout.sources.bench_dir
    cargo_build(bench_dir, ["--release"], label=binary_name, target_dir=target_dir)
    binary = _resolve_target_dir(bench_dir, target_dir) / "release" / binary_name

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
        validate_plugin_dict(data, source=binary_name)
    finally:
        tmp_out.unlink(missing_ok=True)

    return data
