"""Metadata-size fitness plugin."""

from __future__ import annotations

from pathlib import Path

from evolve.fitness._contract import parse_plugin_stdout
from evolve.fitness.plugins._common import cargo_build, run_binary

RUN_TIMEOUT_S = 60


def compute(*, repo_root: Path, target_dir: Path | None = None) -> dict:
    """Runs the metadata-size plugin and returns the validated record."""

    cargo_build(repo_root, ["--bin", "evolve-metadata-size"], label="evolve-metadata-size", target_dir=target_dir)
    binary = (target_dir or repo_root / "target") / "debug" / "evolve-metadata-size"

    result = run_binary([str(binary)], label="evolve-metadata-size", timeout=RUN_TIMEOUT_S)
    return parse_plugin_stdout(result.stdout, source="evolve-metadata-size")
