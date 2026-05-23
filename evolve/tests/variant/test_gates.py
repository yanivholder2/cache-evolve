"""Tests for src/variant/build.py and src/variant/gate_check.py.

Cargo is mocked via tests/_mocks/cargo — the shim short-circuits build/test
calls based on markers in the variant source so gate logic runs end-to-end
without paying for real Rust compilation.

Each test stages a fixture into a Layout-shaped scratch dir so the gate code
exercises the same `Variant.resolve(layout)` path as production.
"""

import re
import shutil
import subprocess
from pathlib import Path

import pytest

from src._io import read_utf8, write_utf8
from src.variant.build import build_variant_for_gates
from src.variant.gate_check import run_gates
from src.identifiers import CycleId, VariantId
from src.paths import CycleVariant, Layout

EVOLVE_ROOT = Path(__file__).parent.parent.parent
FIXTURES_DIR = Path(__file__).parent.parent / "fixtures"
REPO_ROOT = EVOLVE_ROOT.parent

VARIANT = CycleVariant(CycleId(1), VariantId(1))
EXPECTED_VARIANT_TAG = "variant=cycle/0001/v001"


def _stage(tmp_path: Path, fixture_name: str) -> tuple[Layout, CycleVariant]:
    """Stage a fixture into a Layout-shaped scratch dir under tmp_path so
    `VARIANT.resolve(layout)` finds it."""

    layout = Layout(evolve_root=tmp_path / "evolve", outputs_root=tmp_path / "out")
    layout.evolve_root.mkdir(parents=True, exist_ok=True)
    layout.harness_dir.symlink_to(EVOLVE_ROOT / "harness")
    variant_dir = layout.cycle(VARIANT.cycle_id).variant(VARIANT.variant_id)
    shutil.copytree(FIXTURES_DIR / fixture_name / "impl", variant_dir.impl_dir)
    return layout, VARIANT


def test_build_good_fixture_returns_workspace(tmp_path):
    layout, variant = _stage(tmp_path, "good")
    workspace = build_variant_for_gates(variant, layout=layout, repo_root=REPO_ROOT, rejection_log=layout.rejection_log)
    assert workspace.root.is_dir()
    shutil.rmtree(workspace.root, ignore_errors=True)


def test_build_broken_fixture_raises(tmp_path):
    layout, variant = _stage(tmp_path, "broken")
    with pytest.raises(subprocess.CalledProcessError):
        build_variant_for_gates(variant, layout=layout, repo_root=REPO_ROOT, rejection_log=layout.rejection_log)


def test_build_broken_fixture_logs_structured_entry(tmp_path):
    layout, variant = _stage(tmp_path, "broken")
    with pytest.raises(subprocess.CalledProcessError):
        build_variant_for_gates(variant, layout=layout, repo_root=REPO_ROOT, rejection_log=layout.rejection_log)
    content = read_utf8(layout.rejection_log)
    assert "G1 (build)" in content
    assert EXPECTED_VARIANT_TAG in content


def test_gate_check_good_fixture_passes(tmp_path):
    layout, variant = _stage(tmp_path, "good")
    assert run_gates(variant, layout=layout, repo_root=REPO_ROOT, rejection_log=layout.rejection_log)


def test_gate_check_good_fixture_produces_no_rejection_entry(tmp_path):
    layout, variant = _stage(tmp_path, "good")
    run_gates(variant, layout=layout, repo_root=REPO_ROOT, rejection_log=layout.rejection_log)
    log = layout.rejection_log
    assert not log.is_file() or read_utf8(log).strip() == ""


def test_gate_check_broken_fixture_fails(tmp_path):
    layout, variant = _stage(tmp_path, "broken")
    assert not run_gates(variant, layout=layout, repo_root=REPO_ROOT, rejection_log=layout.rejection_log)


def test_gate_check_broken_fixture_logs_structured_entry(tmp_path):
    layout, variant = _stage(tmp_path, "broken")
    run_gates(variant, layout=layout, repo_root=REPO_ROOT, rejection_log=layout.rejection_log)
    content = read_utf8(layout.rejection_log)
    assert "G1 (build)" in content
    assert EXPECTED_VARIANT_TAG in content


def test_gate_check_invalid_ordinal_fails_g2(tmp_path):
    layout, variant = _stage(tmp_path, "good")
    write_utf8(
        variant.resolve(layout).manifest_toml,
        "[variant]\neviction_policy_ordinal = 99\n",
    )
    assert not run_gates(variant, layout=layout, repo_root=REPO_ROOT, rejection_log=layout.rejection_log)
    content = read_utf8(layout.rejection_log)
    assert "G2 (trait contract)" in content
    assert "99" in content


def test_gate_check_bad_invariant_fixture_fails_g3(tmp_path):
    layout, variant = _stage(tmp_path, "bad-invariant")
    result = run_gates(variant, layout=layout, repo_root=REPO_ROOT, rejection_log=layout.rejection_log)
    assert result.g1_passed is True
    assert result.g2_passed is True
    assert result.g3_passed is False
    assert not result.all_passed


def test_gate_check_bad_invariant_fixture_logs_failing_invariant(tmp_path):
    layout, variant = _stage(tmp_path, "bad-invariant")
    run_gates(variant, layout=layout, repo_root=REPO_ROOT, rejection_log=layout.rejection_log)
    content = read_utf8(layout.rejection_log)
    assert "G3 (invariant suite)" in content
    assert EXPECTED_VARIANT_TAG in content
    assert "Fingerprint" in content
    # **Failing invariant** must name a P[1-5] property, not just appear in
    # the embedded panic block.
    assert re.search(r"\*\*Failing invariant\*\*: P[1-5] [a-z_]+", content)


def test_gate_check_good_fixture_passes_g3(tmp_path):
    layout, variant = _stage(tmp_path, "good")
    result = run_gates(variant, layout=layout, repo_root=REPO_ROOT, rejection_log=layout.rejection_log)
    assert result.g1_passed is True
    assert result.g2_passed is True
    assert result.g3_passed is True
    assert result.all_passed
    assert not layout.rejection_log.is_file() or read_utf8(layout.rejection_log).strip() == ""
