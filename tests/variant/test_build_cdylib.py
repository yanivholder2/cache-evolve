"""Tests for src/variant/build.py compile_cdylib (real cargo build, marked slow)."""

import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

import pytest

from evolve.variant.build import compile_cdylib
from evolve.identifiers import CycleId, VariantId
from evolve.paths import CycleVariant, Layout

REPO_ROOT = Path(__file__).parent.parent.parent
FIXTURES_DIR = Path(__file__).parent.parent / "fixtures"
# Pinned to dash, not the swept `config.toml` subject: this suite asserts
# dash-specific JNI symbols (DashRustPolicy) and overlays dash source files,
# so it must build dash whatever subject a run currently selects.
SUBJECT = "dash"

VARIANT = CycleVariant(CycleId(1), VariantId(1))
VARIANT_SYMBOL_PREFIX = (
    "Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_"
    "DashRustPolicy_"
)
# Java side resolves these by JNI naming convention; missing any of them =
# ClassNotFoundException at simulator startup.
EXPECTED_METHODS = (
    "initCache",
    "initDefaultCache",
    "getFromCacheIfPresent",
    "putToCache",
    "dropCache",
)


def _stage(tmp_path: Path, fixture_name: str | None) -> tuple[Layout, CycleVariant]:
    """Stage a Layout pointing at the real subject + framework crates. Optional
    `fixture_name` selects an overlay (None = empty)."""

    layout = Layout(repo_root=tmp_path / "repo", outputs_root=tmp_path / "out", subject=SUBJECT)
    layout.repo_root.mkdir(parents=True, exist_ok=True)
    layout.sources.subject_dir.parent.mkdir(parents=True, exist_ok=True)
    layout.sources.subject_dir.symlink_to(REPO_ROOT / "inputs" / "subjects" / SUBJECT)
    layout.sources.framework_crate_dir.symlink_to(REPO_ROOT / "framework-rs")
    variant_dir = layout.outputs.cycle(VARIANT.cycle_id).variant(VARIANT.variant_id)
    if fixture_name is None:
        variant_dir.overlay_dir.mkdir(parents=True, exist_ok=True)
    else:
        shutil.copytree(FIXTURES_DIR / fixture_name / "files", variant_dir.overlay_dir)
    return layout, VARIANT


def _exported_jni_methods(cdylib_path: Path) -> set[str]:
    nm = subprocess.run(
        ["nm", "-D", "--defined-only", str(cdylib_path)],
        capture_output=True,
        text=True,
        check=True,
    )
    return {
        line.split()[-1].removeprefix(VARIANT_SYMBOL_PREFIX)
        for line in nm.stdout.splitlines()
        if VARIANT_SYMBOL_PREFIX in line
    }


@pytest.mark.slow
def test_build_cdylib_empty_overlay(tmp_path):
    """Baseline-equivalent build: empty overlay produces the subject cdylib
    with all `DashRustPolicy` JNI symbols exported."""

    layout, variant = _stage(tmp_path, None)
    workspace, cdylib_path = compile_cdylib(variant, layout=layout, repo_root=REPO_ROOT)
    try:
        assert cdylib_path.is_file()
        assert cdylib_path.stat().st_size > 0

        if sys.platform == "linux":
            methods = _exported_jni_methods(cdylib_path)
            assert set(EXPECTED_METHODS).issubset(methods), (
                f"variant cdylib missing methods: "
                f"{set(EXPECTED_METHODS) - methods}; exported: {sorted(methods)}"
            )
    finally:
        shutil.rmtree(workspace.root, ignore_errors=True)


@pytest.mark.slow
def test_build_cdylib_broken_overlay_raises_and_produces_no_artifact(tmp_path, monkeypatch):
    layout, variant = _stage(tmp_path, "broken")

    workspaces_root = tmp_path / "cdylib-workspaces"
    workspaces_root.mkdir()
    monkeypatch.setattr(
        "evolve.variant.build.mkdtemp",
        lambda prefix="tmp": tempfile.mkdtemp(prefix=prefix, dir=workspaces_root),
    )

    with pytest.raises(subprocess.CalledProcessError):
        compile_cdylib(variant, layout=layout, repo_root=REPO_ROOT)

    assert list(workspaces_root.iterdir()) == []
    # cdylib failure must NOT write a variant rejection.md (asymmetric vs gates).
    rejection_md = variant.resolve(layout).rejection_md
    assert not rejection_md.exists() or rejection_md.read_text() == ""
