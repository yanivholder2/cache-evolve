"""Tests for src/variant/build.py and src/variant/gates/.

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
from src.paths import CycleVariant, Layout, SourcesLayout, VariantDir

EVOLVE_ROOT = Path(__file__).parent.parent.parent
FIXTURES_DIR = Path(__file__).parent.parent / "fixtures"
REPO_ROOT = EVOLVE_ROOT.parent

VARIANT = CycleVariant(CycleId(1), VariantId(1))
EXPECTED_VARIANT_TAG = "variant=cycle/0001/v001"
ATTEMPT = 1


def _stage(tmp_path: Path, fixture_name: str, *, attempt: int = ATTEMPT) -> tuple[Layout, CycleVariant]:
    """Stage a fixture into a Layout-shaped scratch dir under tmp_path so
    `VARIANT.resolve(layout).attempt(<attempt>).impl_dir` finds it."""

    layout = Layout(evolve_root=tmp_path / "evolve", outputs_root=tmp_path / "out")
    layout.evolve_root.mkdir(parents=True, exist_ok=True)
    layout.sources.rust_scaffold_dir.parent.mkdir(parents=True, exist_ok=True)
    layout.sources.rust_scaffold_dir.symlink_to(SourcesLayout(root=EVOLVE_ROOT).rust_scaffold_dir)
    variant_dir = layout.outputs.cycle(VARIANT.cycle_id).variant(VARIANT.variant_id)
    attempt_dir = variant_dir.attempt(attempt)
    shutil.copytree(VariantDir(root=FIXTURES_DIR / fixture_name).impl_dir, attempt_dir.impl_dir)
    return layout, VARIANT


def test_build_good_fixture_returns_workspace(tmp_path):
    layout, variant = _stage(tmp_path, "good")
    workspace = build_variant_for_gates(variant, layout=layout, repo_root=REPO_ROOT)
    assert workspace.root.is_dir()
    shutil.rmtree(workspace.root, ignore_errors=True)


def test_build_broken_fixture_raises(tmp_path):
    layout, variant = _stage(tmp_path, "broken")
    with pytest.raises(subprocess.CalledProcessError):
        build_variant_for_gates(variant, layout=layout, repo_root=REPO_ROOT)


def test_gate_check_good_fixture_passes(tmp_path):
    layout, variant = _stage(tmp_path, "good")
    assert run_gates(variant, attempt=ATTEMPT, layout=layout, repo_root=REPO_ROOT)


def test_gate_check_good_fixture_produces_no_rejection_entry(tmp_path):
    layout, variant = _stage(tmp_path, "good")
    run_gates(variant, attempt=ATTEMPT, layout=layout, repo_root=REPO_ROOT)
    rejection_md = variant.resolve(layout).rejection_md
    assert not rejection_md.is_file() or read_utf8(rejection_md).strip() == ""


def test_gate_check_broken_fixture_fails(tmp_path):
    layout, variant = _stage(tmp_path, "broken")
    assert not run_gates(variant, attempt=ATTEMPT, layout=layout, repo_root=REPO_ROOT)


def test_gate_check_broken_fixture_logs_structured_entry(tmp_path):
    layout, variant = _stage(tmp_path, "broken")
    run_gates(variant, attempt=ATTEMPT, layout=layout, repo_root=REPO_ROOT)
    content = read_utf8(variant.resolve(layout).rejection_md)
    assert "build gate" in content
    assert EXPECTED_VARIANT_TAG in content
    assert "attempt=1" in content


def test_gate_check_invalid_policy_name_fails_contract_gate(tmp_path):
    layout, variant = _stage(tmp_path, "good")
    write_utf8(
        variant.resolve(layout).manifest_toml,
        "[variant]\neviction_policy = \"NotAPolicy\"\n",
    )
    assert not run_gates(variant, attempt=ATTEMPT, layout=layout, repo_root=REPO_ROOT)
    content = read_utf8(variant.resolve(layout).rejection_md)
    assert "contract gate" in content
    assert "NotAPolicy" in content


def test_gate_check_bad_invariant_fixture_fails_invariant_gate(tmp_path):
    layout, variant = _stage(tmp_path, "bad-invariant")
    result = run_gates(variant, attempt=ATTEMPT, layout=layout, repo_root=REPO_ROOT)
    assert result.outcomes["build gate"].passed is True
    assert result.outcomes["contract gate"].passed is True
    assert result.outcomes["invariant gate"].passed is False
    assert not result.all_passed
    assert result.first_failed_gate == "invariant gate"


def test_gate_check_bad_invariant_fixture_logs_failing_invariant(tmp_path):
    layout, variant = _stage(tmp_path, "bad-invariant")
    run_gates(variant, attempt=ATTEMPT, layout=layout, repo_root=REPO_ROOT)
    content = read_utf8(variant.resolve(layout).rejection_md)
    assert "invariant gate" in content
    assert EXPECTED_VARIANT_TAG in content
    assert re.search(r"\*\*Failing invariant\*\*: P[1-5] [a-z_]+", content)


def test_gate_check_good_fixture_passes_all_gates(tmp_path):
    layout, variant = _stage(tmp_path, "good")
    result = run_gates(variant, attempt=ATTEMPT, layout=layout, repo_root=REPO_ROOT)
    assert result.outcomes["build gate"].passed is True
    assert result.outcomes["contract gate"].passed is True
    assert result.outcomes["invariant gate"].passed is True
    assert result.all_passed
    assert result.first_failed_gate is None
    rejection_md = variant.resolve(layout).rejection_md
    assert not rejection_md.is_file() or read_utf8(rejection_md).strip() == ""


def test_gate_check_rejection_header_records_attempt_number(tmp_path):
    layout, variant = _stage(tmp_path, "bad-invariant", attempt=2)
    run_gates(variant, attempt=2, layout=layout, repo_root=REPO_ROOT)
    content = read_utf8(variant.resolve(layout).rejection_md)
    assert "attempt=2" in content
