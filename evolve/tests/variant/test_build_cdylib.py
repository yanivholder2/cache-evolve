"""Tests for src/variant/build.py build_cdylib (real cargo build, marked slow)."""

import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

import pytest

from src._io import write_utf8
from src.variant.build import build_cdylib
from src.identifiers import CycleId, VariantId
from src.paths import CycleVariant, Layout

EVOLVE_ROOT = Path(__file__).parent.parent.parent
REPO_ROOT = EVOLVE_ROOT.parent

VARIANT = CycleVariant(CycleId(1), VariantId(1))
VARIANT_SYMBOL_PREFIX = (
    "Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_"
    "DashVariantRustPolicy_"
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


def _stage_baseline(tmp_path: Path) -> tuple[Layout, CycleVariant]:
    layout = Layout(evolve_root=tmp_path / "evolve", outputs_root=tmp_path / "out")
    layout.evolve_root.mkdir(parents=True, exist_ok=True)
    layout.sources.caffeine_jni_dir.symlink_to(EVOLVE_ROOT / "caffeine_jni")
    variant_dir = layout.cycle(VARIANT.cycle_id).variant(VARIANT.variant_id)
    shutil.copytree(EVOLVE_ROOT / "subject" / "baseline" / "impl", variant_dir.impl_dir)
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
def test_build_cdylib_good_baseline(tmp_path):
    layout, variant = _stage_baseline(tmp_path)
    workspace, cdylib_path = build_cdylib(variant, layout=layout, repo_root=REPO_ROOT)
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
def test_build_cdylib_broken_variant_raises_and_produces_no_artifact(tmp_path, monkeypatch):
    layout, variant = _stage_baseline(tmp_path)
    write_utf8(variant.resolve(layout).segment_rs, "this is not valid rust;\n")

    workspaces_root = tmp_path / "cdylib-workspaces"
    workspaces_root.mkdir()
    rejection_log = tmp_path / "rejection-log.md"
    monkeypatch.setattr(
        "src.paths.mkdtemp",
        lambda prefix="tmp": tempfile.mkdtemp(prefix=prefix, dir=workspaces_root),
    )

    with pytest.raises(subprocess.CalledProcessError):
        build_cdylib(variant, layout=layout, repo_root=REPO_ROOT)

    assert list(workspaces_root.iterdir()) == []
    # cdylib failure must NOT touch the rejection log (asymmetric vs gates).
    assert not rejection_log.exists() or rejection_log.read_text() == ""
