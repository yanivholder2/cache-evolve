"""Metadata-size fitness plugin.

Runs the evolve-metadata-size binary, validates its output against the plugin
contract, and returns the parsed record.
"""

from __future__ import annotations

import sys
from pathlib import Path

_HERE = Path(__file__).resolve()
sys.path.insert(0, str(_HERE.parent))

from _common import cargo_build, run_binary  # noqa: E402
from _contract import parse_plugin_stdout  # noqa: E402

RUN_TIMEOUT_S = 60


def compute(*, repo_root: Path, target_dir: Path | None = None) -> dict:
    """Build + run the metadata-size plugin; return the validated record."""

    cargo_build(repo_root, ["--bin", "evolve-metadata-size"], label="evolve-metadata-size", target_dir=target_dir)
    binary = (target_dir or repo_root / "target") / "debug" / "evolve-metadata-size"

    result = run_binary([str(binary)], label="evolve-metadata-size", timeout=RUN_TIMEOUT_S)
    return parse_plugin_stdout(result.stdout, source="evolve-metadata-size")
