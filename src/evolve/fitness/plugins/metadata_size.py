"""Metadata-size fitness plugin."""

from __future__ import annotations

from pathlib import Path

from evolve.fitness._contract import parse_plugin_stdout
from evolve.fitness.plugins._common import cargo_build, run_binary

RUN_TIMEOUT_S = 60


def compute(*, repo_root: Path, package: str, binary_name: str, target_dir: Path | None = None) -> dict:
    """Runs the selected subject's metadata-size bin and returns the validated record.

    The subject must declare a `[metadata]` table in its manifest (`runner` only
    schedules this plugin when it does). The package selector ensures only the
    selected subject's bin compiles, avoiding output-path collisions between
    subjects that share a bin name."""

    cargo_build(repo_root, ["-p", package, "--bin", binary_name], label=binary_name, target_dir=target_dir)
    binary = (target_dir or repo_root / "target") / "debug" / binary_name

    result = run_binary([str(binary)], label=binary_name, timeout=RUN_TIMEOUT_S)
    return parse_plugin_stdout(result.stdout, source=binary_name)
