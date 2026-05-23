"""G1 gate: compile a variant impl/ into a test artifact."""

from __future__ import annotations

import logging
import shutil
import subprocess
import tempfile
from datetime import datetime, timezone
from pathlib import Path

from src.paths import Variant

log = logging.getLogger("build")

HARNESS_DIR = Path(__file__).resolve().parent.parent / "harness"

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


def _scaffold_workspace(work_dir: Path, impl_dir: Path, repo_root: Path) -> None:
    (work_dir / "src").mkdir(parents=True, exist_ok=True)
    (work_dir / "tests").mkdir(parents=True, exist_ok=True)
    (work_dir / "Cargo.toml").write_text(_cargo_toml(repo_root), encoding="utf-8")
    (work_dir / "src" / "lib.rs").write_text(LIB_RS, encoding="utf-8")
    shutil.copy(impl_dir / "segment.rs", work_dir / "src" / "variant.rs")
    manifest = impl_dir / "manifest.toml"
    if manifest.is_file():
        shutil.copy(manifest, work_dir / "manifest.toml")
    shutil.copy(HARNESS_DIR / "g2_contract.rs", work_dir / "tests" / "g2_contract.rs")
    shutil.copy(HARNESS_DIR / "g3_invariant.rs", work_dir / "tests" / "g3_invariant.rs")


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
    with open(rejection_log, "a", encoding="utf-8") as f:
        f.write(entry)


def build_variant(
    impl_dir: Path,
    variant: Variant,
    *,
    repo_root: Path,
    rejection_log: Path,
) -> Path:
    """Compile a variant impl into a temp workspace. Returns the workspace path.

    Raises:
        FileNotFoundError: impl_dir is missing segment.rs.
        subprocess.CalledProcessError: cargo build failed (rejection appended
            to rejection_log, workspace removed).
    """

    impl_dir = impl_dir.resolve()
    segment_rs = impl_dir / "segment.rs"
    if not segment_rs.is_file():
        raise FileNotFoundError(f"{segment_rs} not found — impl/ must contain segment.rs")

    work_dir = Path(tempfile.mkdtemp(prefix="dash-variant-"))
    log.debug("workspace=%s impl=%s repo_root=%s", work_dir, impl_dir, repo_root)
    _scaffold_workspace(work_dir, impl_dir, repo_root)

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
