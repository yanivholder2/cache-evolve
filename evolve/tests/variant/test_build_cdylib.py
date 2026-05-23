"""Tests for src/variant/build.py build_cdylib (real cargo build, marked slow)."""

import shutil
import subprocess
import sys
from pathlib import Path

import pytest

from src.variant.build import build_cdylib
from src.identifiers import CycleId, VariantId
from src.paths import CycleVariant, Layout

EVOLVE_ROOT = Path(__file__).parent.parent.parent
REPO_ROOT = EVOLVE_ROOT.parent

VARIANT = CycleVariant(CycleId(1), VariantId(1))
EXPECTED_SYMBOL = (
    "Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_"
    "DashVariantRustPolicy_initCache"
)


def _stage_baseline(tmp_path: Path) -> tuple[Layout, CycleVariant]:
    layout = Layout(evolve_root=tmp_path / "evolve", outputs_root=tmp_path / "out")
    layout.evolve_root.mkdir(parents=True, exist_ok=True)
    layout.caffeine_jni_dir.symlink_to(EVOLVE_ROOT / "caffeine_jni")
    variant_dir = layout.cycle(VARIANT.cycle_id).variant(VARIANT.variant_id)
    shutil.copytree(EVOLVE_ROOT / "subject" / "baseline" / "impl", variant_dir.impl_dir)
    return layout, VARIANT


@pytest.mark.slow
def test_build_cdylib_good_baseline(tmp_path):
    layout, variant = _stage_baseline(tmp_path)
    cdylib_path = build_cdylib(variant, layout=layout, repo_root=REPO_ROOT)
    work_dir = cdylib_path.parent.parent.parent
    try:
        assert cdylib_path.is_file()
        assert cdylib_path.stat().st_size > 0

        if sys.platform == "linux":
            nm = subprocess.run(
                ["nm", "-D", str(cdylib_path)],
                capture_output=True,
                text=True,
                check=True,
            )
            assert EXPECTED_SYMBOL in nm.stdout, (
                f"missing symbol {EXPECTED_SYMBOL} in nm output:\n{nm.stdout}"
            )
    finally:
        shutil.rmtree(work_dir, ignore_errors=True)
