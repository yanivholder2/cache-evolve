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

from evolve._io import read_utf8
from evolve.variant.build import compile_for_gates
from evolve.variant.gate_check import run_gates
from evolve.identifiers import CycleId, VariantId
from evolve.paths import CycleVariant, Layout

REPO_ROOT = Path(__file__).parent.parent.parent
FIXTURES_DIR = Path(__file__).parent.parent / "fixtures"

VARIANT = CycleVariant(CycleId(1), VariantId(1))
EXPECTED_VARIANT_TAG = "variant=cycle/0001/v001"


def _stage(tmp_path: Path, fixture_name: str) -> tuple[Layout, CycleVariant]:
    """Stage a fixture into a Layout-shaped scratch dir under tmp_path. The
    repo subtree is a thin symlink farm pointing at the real subject + framework
    crate so the build harness's copytree + overlay path resolves to the real
    sources without mutating them."""

    layout = Layout(repo_root=tmp_path / "repo", outputs_root=tmp_path / "out")
    layout.repo_root.mkdir(parents=True, exist_ok=True)
    real = REPO_ROOT
    layout.sources.subject_dir.parent.mkdir(parents=True, exist_ok=True)
    (layout.sources.subject_dir).symlink_to(real / "inputs" / "subjects" / "dash")
    layout.sources.framework_crate_dir.symlink_to(real / "framework-rs")
    variant_dir = layout.outputs.cycle(VARIANT.cycle_id).variant(VARIANT.variant_id)
    fixture_files = FIXTURES_DIR / fixture_name / "files"
    if fixture_files.is_dir():
        shutil.copytree(fixture_files, variant_dir.files_dir)
    else:
        variant_dir.files_dir.mkdir(parents=True, exist_ok=True)
    return layout, VARIANT


def test_build_good_fixture_returns_workspace(tmp_path):
    layout, variant = _stage(tmp_path, "good")
    workspace = compile_for_gates(variant, layout=layout, repo_root=REPO_ROOT)
    assert workspace.root.is_dir()
    shutil.rmtree(workspace.root, ignore_errors=True)


def test_build_broken_fixture_raises(tmp_path):
    layout, variant = _stage(tmp_path, "broken")
    with pytest.raises(subprocess.CalledProcessError):
        compile_for_gates(variant, layout=layout, repo_root=REPO_ROOT)


def test_build_broken_fixture_logs_structured_entry(tmp_path):
    layout, variant = _stage(tmp_path, "broken")
    with pytest.raises(subprocess.CalledProcessError):
        compile_for_gates(variant, layout=layout, repo_root=REPO_ROOT)
    content = read_utf8(variant.resolve(layout).rejection_md)
    assert "build gate" in content
    assert EXPECTED_VARIANT_TAG in content


def test_gate_check_good_fixture_passes(tmp_path):
    layout, variant = _stage(tmp_path, "good")
    assert run_gates(variant, layout=layout, repo_root=REPO_ROOT)


def test_gate_check_good_fixture_produces_no_rejection_entry(tmp_path):
    layout, variant = _stage(tmp_path, "good")
    run_gates(variant, layout=layout, repo_root=REPO_ROOT)
    rejection_md = variant.resolve(layout).rejection_md
    assert not rejection_md.is_file() or read_utf8(rejection_md).strip() == ""


def test_gate_check_broken_fixture_fails(tmp_path):
    layout, variant = _stage(tmp_path, "broken")
    assert not run_gates(variant, layout=layout, repo_root=REPO_ROOT)


def test_gate_check_broken_fixture_logs_structured_entry(tmp_path):
    layout, variant = _stage(tmp_path, "broken")
    run_gates(variant, layout=layout, repo_root=REPO_ROOT)
    content = read_utf8(variant.resolve(layout).rejection_md)
    assert "build gate" in content
    assert EXPECTED_VARIANT_TAG in content


def test_gate_check_bad_invariant_fixture_fails_invariant_gate(tmp_path):
    layout, variant = _stage(tmp_path, "bad-invariant")
    result = run_gates(variant, layout=layout, repo_root=REPO_ROOT)
    assert result.build_gate_passed is True
    assert result.contract_gate_passed is True
    assert result.invariant_gate_passed is False
    assert not result.all_passed


def test_gate_check_bad_invariant_fixture_logs_failing_invariant(tmp_path):
    layout, variant = _stage(tmp_path, "bad-invariant")
    run_gates(variant, layout=layout, repo_root=REPO_ROOT)
    content = read_utf8(variant.resolve(layout).rejection_md)
    assert "invariant gate" in content
    assert EXPECTED_VARIANT_TAG in content
    # **Failing invariant** must name a P[1-5] property, not just appear in
    # the embedded panic block.
    assert re.search(r"\*\*Failing invariant\*\*: P[1-5] [a-z_]+", content)


def test_gate_check_good_fixture_passes_invariant_gate(tmp_path):
    layout, variant = _stage(tmp_path, "good")
    result = run_gates(variant, layout=layout, repo_root=REPO_ROOT)
    assert result.build_gate_passed is True
    assert result.contract_gate_passed is True
    assert result.invariant_gate_passed is True
    assert result.all_passed
    rejection_md = variant.resolve(layout).rejection_md
    assert not rejection_md.is_file() or read_utf8(rejection_md).strip() == ""
