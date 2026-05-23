"""G1 gate: compile a variant impl/ into a test artifact."""

from __future__ import annotations

import logging
import shutil
import subprocess
import tempfile
from datetime import datetime, timezone
from pathlib import Path

from src._io import append_utf8, write_utf8
from src.paths import Layout, Variant, VariantDir

log = logging.getLogger("build")

LIB_RS = "pub mod variant;\npub use variant::VariantSegment;\n"


def _cargo_toml(repo_root: Path) -> str:
    return (
        "[package]\n"
        'name = "variant-check"\n'
        'version = "0.1.0"\n'
        'edition = "2021"\n'
        "\n"
        "[dependencies]\n"
        f'dash = {{ package = "dash-cache-rs", path = "{repo_root}", features = ["test-utils"] }}\n'
    )


def _scaffold_workspace(work_dir: Path, variant_dir: VariantDir, repo_root: Path, harness_dir: Path) -> None:
    """Lay out a cargo crate around the variant's segment.rs so the G2/G3 harness tests can link against it."""

    (work_dir / "src").mkdir(parents=True, exist_ok=True)
    (work_dir / "tests").mkdir(parents=True, exist_ok=True)
    write_utf8(work_dir / "Cargo.toml", _cargo_toml(repo_root))
    write_utf8(work_dir / "src" / "lib.rs", LIB_RS)
    shutil.copy(variant_dir.segment_rs, work_dir / "src" / "variant.rs")
    shutil.copy(variant_dir.manifest_toml, work_dir / "manifest.toml")
    shutil.copy(harness_dir / "g2_contract.rs", work_dir / "tests" / "g2_contract.rs")
    shutil.copy(harness_dir / "g3_invariant.rs", work_dir / "tests" / "g3_invariant.rs")


def _append_rejection(rejection_log: Path, variant: Variant, cargo_output: str) -> None:
    ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    entry = (
        f"\n## {ts} — variant={variant}\n\n"
        f"**Gate**: G1 (build)\n"
        f"**Error**:\n"
        f"```\n{cargo_output}\n```\n"
        f"---\n"
    )
    rejection_log.parent.mkdir(parents=True, exist_ok=True)
    append_utf8(rejection_log, entry)


def build_variant(
    variant: Variant,
    *,
    layout: Layout,
    repo_root: Path,
    rejection_log: Path,
) -> Path:
    """Compile a variant impl into a temp workspace. Returns the workspace path.

    Raises:
        subprocess.CalledProcessError: cargo build failed (rejection appended
            to rejection_log, workspace removed).
    """

    variant_dir = variant.resolve(layout)
    work_dir = Path(tempfile.mkdtemp(prefix="dash-variant-"))
    log.debug("workspace=%s impl=%s repo_root=%s", work_dir, variant_dir.impl_dir, repo_root)
    _scaffold_workspace(work_dir, variant_dir, repo_root, layout.harness_dir)

    log.info("G1: compiling variant %s ...", variant)
    proc = subprocess.run(
        ["cargo", "build", "--tests", "--manifest-path", str(work_dir / "Cargo.toml")],
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        log.info("G1: FAILED")
        _append_rejection(rejection_log, variant, proc.stdout + proc.stderr)
        log.info("rejection logged to %s", rejection_log)
        shutil.rmtree(
            work_dir,
            onexc=lambda fn, path, exception: log.warning("workspace cleanup failed for %s: %s", path, exception),
        )
        raise subprocess.CalledProcessError(proc.returncode, proc.args, proc.stdout, proc.stderr)

    log.info("G1: passed")
    return work_dir
