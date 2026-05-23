"""Variant cargo builds: G1 gate's test artifact (`build_variant_for_gates`) and the
Caffeine-loadable cdylib (`build_cdylib`)."""

from __future__ import annotations

import logging
import shutil
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path

from src._io import append_utf8, write_utf8
from src.paths import CargoWorkspaceDir, Layout, Variant, VariantDir

log = logging.getLogger("build")

LIB_RS_GATES = "pub mod variant;\npub use variant::VariantSegment;\n"
LIB_RS_CDYLIB = "pub mod variant;\npub mod jni;\npub use variant::VariantSegment;\n"

_CDYLIB_FILENAMES = {
    "linux": "libdash_variant.so",
    "win32": "dash_variant.dll",
    "darwin": "libdash_variant.dylib",
}


def _cargo_toml(repo_root: Path, *, with_cdylib: bool = False) -> str:
    base = (
        "[package]\n"
        'name = "variant-check"\n'
        'version = "0.1.0"\n'
        'edition = "2021"\n'
        "\n"
    )
    if with_cdylib:
        base += (
            "[lib]\n"
            'name = "dash_variant"\n'
            'crate-type = ["cdylib", "rlib"]\n'
            "\n"
        )
    base += (
        "[dependencies]\n"
        f'dash = {{ package = "dash-cache-rs", path = "{repo_root}", features = ["test-utils"] }}\n'
    )
    if with_cdylib:
        base += 'jni = "0.20.0"\n'
        base += 'log = "0.4"\n'
    return base


def _scaffold_variant_workspace(
    workspace: CargoWorkspaceDir, variant_dir: VariantDir, *, cargo_toml: str, lib_rs: str
) -> None:
    """Lay out the shared skeleton every variant cargo crate needs: src/lib.rs, src/variant.rs, Cargo.toml, manifest.toml."""

    workspace.src_dir.mkdir(parents=True, exist_ok=True)
    write_utf8(workspace.cargo_toml, cargo_toml)
    write_utf8(workspace.lib_rs, lib_rs)
    shutil.copy(variant_dir.segment_rs, workspace.variant_rs)
    shutil.copy(variant_dir.manifest_toml, workspace.manifest_toml)


def _scaffold_gates_workspace(workspace: CargoWorkspaceDir, variant_dir: VariantDir, layout: Layout, repo_root: Path) -> None:
    """Lay out a cargo crate around the variant's segment.rs so the G2/G3 harness tests can link against it."""

    _scaffold_variant_workspace(workspace, variant_dir, cargo_toml=_cargo_toml(repo_root), lib_rs=LIB_RS_GATES)
    workspace.tests_dir.mkdir(parents=True, exist_ok=True)
    shutil.copy(layout.g2_contract_rs, workspace.g2_contract_rs)
    shutil.copy(layout.g3_invariant_rs, workspace.g3_invariant_rs)


def _scaffold_cdylib_workspace(
    workspace: CargoWorkspaceDir, variant_dir: VariantDir, layout: Layout, repo_root: Path
) -> None:
    """Lay out a cargo crate around the variant's segment.rs plus the Caffeine JNI shim, ready to be built as a cdylib."""

    _scaffold_variant_workspace(
        workspace, variant_dir, cargo_toml=_cargo_toml(repo_root, with_cdylib=True), lib_rs=LIB_RS_CDYLIB
    )
    shutil.copy(layout.variant_jni_rs, workspace.jni_rs)


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


def build_variant_for_gates(
    variant: Variant,
    *,
    layout: Layout,
    repo_root: Path,
    rejection_log: Path,
) -> CargoWorkspaceDir:
    """Compile a variant impl into a temp workspace. Returns the workspace.

    Raises:
        subprocess.CalledProcessError: cargo build failed (rejection appended
            to rejection_log, workspace removed).
    """

    variant_dir = variant.resolve(layout)
    workspace = CargoWorkspaceDir(root=Path(tempfile.mkdtemp(prefix="dash-variant-")))
    log.debug("workspace=%s impl=%s repo_root=%s", workspace.root, variant_dir.impl_dir, repo_root)
    _scaffold_gates_workspace(workspace, variant_dir, layout, repo_root)

    log.info("G1: compiling variant %s ...", variant)
    proc = subprocess.run(
        ["cargo", "build", "--tests", "--manifest-path", str(workspace.cargo_toml)],
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        log.info("G1: FAILED")
        _append_rejection(rejection_log, variant, proc.stdout + proc.stderr)
        log.info("rejection logged to %s", rejection_log)
        shutil.rmtree(
            workspace.root,
            onexc=lambda fn, path, exception: log.warning("workspace cleanup failed for %s: %s", path, exception),
        )
        raise subprocess.CalledProcessError(proc.returncode, proc.args, proc.stdout, proc.stderr)

    log.info("G1: passed")
    return workspace


def build_cdylib(variant: Variant, *, layout: Layout, repo_root: Path) -> Path:
    """Compile a variant into a Caffeine-loadable cdylib. Returns the absolute path to the artifact.

    Raises:
        subprocess.CalledProcessError: cargo build failed (workspace removed).
    """

    filename = _CDYLIB_FILENAMES[sys.platform]
    variant_dir = variant.resolve(layout)
    workspace = CargoWorkspaceDir(root=Path(tempfile.mkdtemp(prefix="dash-variant-cdylib-")))
    log.debug("cdylib workspace=%s impl=%s repo_root=%s", workspace.root, variant_dir.impl_dir, repo_root)
    _scaffold_cdylib_workspace(workspace, variant_dir, layout, repo_root)

    log.info("building cdylib for variant %s ...", variant)
    proc = subprocess.run(
        ["cargo", "build", "--release", "--manifest-path", str(workspace.cargo_toml)],
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        log.info("cdylib build FAILED for %s", variant)
        shutil.rmtree(
            workspace.root,
            onexc=lambda fn, path, exception: log.warning("workspace cleanup failed for %s: %s", path, exception),
        )
        raise subprocess.CalledProcessError(proc.returncode, proc.args, proc.stdout, proc.stderr)

    cdylib_path = workspace.cdylib(filename)
    log.info("cdylib built at %s", cdylib_path)
    return cdylib_path
