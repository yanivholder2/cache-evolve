"""Metadata-size fitness plugin."""

from __future__ import annotations

import logging
import shutil
from pathlib import Path

from evolve.fitness._contract import parse_plugin_stdout
from evolve.fitness.plugins._common import cargo_build, run_binary
from evolve.paths import Layout, Variant
from evolve.variant.build import prepare_variant_workspace

log = logging.getLogger(__name__)

RUN_TIMEOUT_S = 60


def compute(*, variant: Variant, layout: Layout, repo_root: Path, binary_name: str) -> dict:
    """Runs the metadata-size bin against the variant's overlaid workspace and
    returns the validated record.

    The bin is built and run inside a per-variant workspace (subject copytree +
    `overlay/`), mirroring the cdylib build, so `size_of` reflects the variant's
    own type edits rather than the canonical subject types. The subject must
    declare a `[metadata]` table in its manifest (`runner` only schedules this
    plugin when it does)."""

    workspace = prepare_variant_workspace(variant, layout=layout, repo_root=repo_root)
    try:
        cargo_build(workspace.root, ["--bin", binary_name], label=binary_name)
        binary = workspace.root / "target" / "debug" / binary_name
        result = run_binary([str(binary)], label=binary_name, timeout=RUN_TIMEOUT_S)
        return parse_plugin_stdout(result.stdout, source=binary_name)
    finally:
        shutil.rmtree(
            workspace.root,
            onexc=lambda fn, path, exception: log.warning(
                "workspace cleanup failed for %s: %s", path, exception
            ),
        )
