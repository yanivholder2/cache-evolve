"""Variant cargo builds: the build gate's test artifact and the
Caffeine-loadable cdylib for one variant."""

from __future__ import annotations

import logging
import shutil
import subprocess
from datetime import datetime, timezone
from pathlib import Path

from src._io import append_utf8, write_utf8
from src.paths import (
    CargoWorkspaceDir,
    CdylibWorkspace,
    GatesWorkspace,
    Layout,
    Variant,
    VariantDir,
    cargo_workspace_tempdir,
)

log = logging.getLogger("build")

LIB_RS_GATES = "pub mod variant;\npub use variant::VariantSegment;\n"
LIB_RS_CDYLIB = "pub mod variant;\npub mod jni;\npub use variant::VariantSegment;\n"

_JNI_CRATE_VERSION = "0.20.0"
_LOG_CRATE_VERSION = "0.4"


def _cargo_toml_gates(repo_root: Path) -> str:
    repo_root_str = repo_root.as_posix()
    return (
        "[package]\n"
        'name = "variant-check"\n'
        'version = "0.1.0"\n'
        'edition = "2021"\n'
        "\n"
        "[dependencies]\n"
        f'dash = {{ package = "dash-cache-rs", path = "{repo_root_str}", '
        'features = ["test-utils"] }\n'
    )


def _cargo_toml_cdylib(repo_root: Path) -> str:
    # Production cdylib must not pull `test-utils` (proptest, test helpers).
    repo_root_str = repo_root.as_posix()
    return (
        "[package]\n"
        'name = "variant-check"\n'
        'version = "0.1.0"\n'
        'edition = "2021"\n'
        "\n"
        "[lib]\n"
        'name = "dash_variant"\n'
        'crate-type = ["cdylib", "rlib"]\n'
        "\n"
        "[dependencies]\n"
        f'dash = {{ package = "dash-cache-rs", path = "{repo_root_str}" }}\n'
        f'jni = "{_JNI_CRATE_VERSION}"\n'
        f'log = "{_LOG_CRATE_VERSION}"\n'
    )


def _scaffold_variant_workspace(
    workspace: CargoWorkspaceDir, variant_dir: VariantDir, *, cargo_toml: str, lib_rs: str
) -> None:
    workspace.src_dir.mkdir(parents=True, exist_ok=True)
    write_utf8(workspace.cargo_toml, cargo_toml)
    write_utf8(workspace.lib_rs, lib_rs)
    shutil.copy(variant_dir.segment_rs, workspace.variant_rs)
    shutil.copy(variant_dir.manifest_toml, workspace.manifest_toml)


def _scaffold_gates_workspace(workspace: GatesWorkspace, variant_dir: VariantDir, layout: Layout, repo_root: Path) -> None:
    _scaffold_variant_workspace(workspace, variant_dir, cargo_toml=_cargo_toml_gates(repo_root), lib_rs=LIB_RS_GATES)
    workspace.tests_dir.mkdir(parents=True, exist_ok=True)
    shutil.copy(layout.sources.contract_gate_rs, workspace.contract_gate_rs)
    shutil.copy(layout.sources.invariant_gate_rs, workspace.invariant_gate_rs)


def _scaffold_cdylib_workspace(
    workspace: CdylibWorkspace, variant_dir: VariantDir, layout: Layout, repo_root: Path
) -> None:
    _scaffold_variant_workspace(
        workspace, variant_dir, cargo_toml=_cargo_toml_cdylib(repo_root), lib_rs=LIB_RS_CDYLIB
    )
    shutil.copy(layout.sources.variant_jni_rs, workspace.jni_rs)


def _append_rejection(rejection_md: Path, variant: Variant, cargo_output: str) -> None:
    ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    entry = (
        f"\n## {ts} — variant={variant}\n\n"
        f"**Gate**: build gate\n"
        f"**Error**:\n"
        f"```\n{cargo_output}\n```\n"
        f"---\n"
    )
    rejection_md.parent.mkdir(parents=True, exist_ok=True)
    append_utf8(rejection_md, entry)


def build_variant_for_gates(
    variant: Variant,
    *,
    layout: Layout,
    repo_root: Path,
) -> GatesWorkspace:
    """Compile a variant impl into a temp workspace. Returns the workspace.

    Raises:
        subprocess.CalledProcessError: cargo build failed (rejection appended
            to the variant's rejection.md, workspace removed).
    """

    variant_dir = variant.resolve(layout)
    rejection_md = variant_dir.rejection_md
    with cargo_workspace_tempdir("dash-variant-", GatesWorkspace) as workspace:
        log.debug("workspace=%s impl=%s repo_root=%s", workspace.root, variant_dir.impl_dir, repo_root)
        _scaffold_gates_workspace(workspace, variant_dir, layout, repo_root)

        log.info("build gate: compiling variant %s ...", variant)
        proc = subprocess.run(
            ["cargo", "build", "--tests", "--manifest-path", str(workspace.cargo_toml)],
            capture_output=True,
            text=True,
        )
        if proc.returncode != 0:
            log.info("build gate: FAILED")
            _append_rejection(rejection_md, variant, proc.stdout + proc.stderr)
            log.info("rejection logged to %s", rejection_md)
            raise subprocess.CalledProcessError(proc.returncode, proc.args, proc.stdout, proc.stderr)

        log.info("build gate: passed")
        # Caller owns workspace from here; the CM only cleans up on exception.
        return workspace


def build_cdylib(
    variant: Variant, *, layout: Layout, repo_root: Path
) -> tuple[CdylibWorkspace, Path]:
    """Compile a variant into a Caffeine-loadable cdylib.

    Returns a `(workspace, cdylib_path)` tuple. The caller owns the workspace's
    lifecycle and is responsible for cleaning it up (e.g. `shutil.rmtree(workspace.root)`)
    once the artifact is no longer needed.

    Raises:
        RuntimeError: platform is unsupported.
        subprocess.CalledProcessError: cargo build failed (workspace removed).
    """

    variant_dir = variant.resolve(layout)
    with cargo_workspace_tempdir("dash-variant-cdylib-", CdylibWorkspace) as workspace:
        log.debug("cdylib workspace=%s impl=%s repo_root=%s", workspace.root, variant_dir.impl_dir, repo_root)
        _scaffold_cdylib_workspace(workspace, variant_dir, layout, repo_root)

        log.info("building cdylib for variant %s ...", variant)
        proc = subprocess.run(
            ["cargo", "build", "--release", "--manifest-path", str(workspace.cargo_toml)],
            capture_output=True,
            text=True,
        )
        if proc.returncode != 0:
            log.error("cdylib build FAILED for %s\n%s", variant, proc.stdout + proc.stderr)
            raise subprocess.CalledProcessError(proc.returncode, proc.args, proc.stdout, proc.stderr)

        cdylib_path = workspace.cdylib
        log.info("cdylib built at %s", cdylib_path)
        return workspace, cdylib_path
