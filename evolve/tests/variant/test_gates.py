"""Tests for src/variant/build.py and src/variant/gate_check.py.

Cargo is mocked via tests/_mocks/cargo — the shim short-circuits build/test
calls based on markers in the variant source so gate logic runs end-to-end
without paying for real Rust compilation.
"""

import re
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

from src.variant.build import build_variant
from src.variant.gate_check import run_gates
from src.identifiers import CycleId, VariantId
from src.paths import CycleVariant

EVOLVE_ROOT = Path(__file__).parent.parent.parent
FIXTURES_DIR = Path(__file__).parent.parent / "fixtures"
REPO_ROOT = EVOLVE_ROOT.parent

GOOD_FIXTURE = FIXTURES_DIR / "good" / "impl"
BROKEN_FIXTURE = FIXTURES_DIR / "broken" / "impl"
BAD_INVARIANT_FIXTURE = FIXTURES_DIR / "bad-invariant" / "impl"

VARIANT = CycleVariant(CycleId(1), VariantId(1))
EXPECTED_VARIANT_TAG = "variant=cycle/0001/v001"


def _rejection_log(tmp_path) -> Path:
    return tmp_path / "rejection-log.md"


def test_build_good_fixture_returns_workspace(tmp_path):
    work = build_variant(GOOD_FIXTURE, VARIANT, repo_root=REPO_ROOT, rejection_log=_rejection_log(tmp_path))
    assert work.is_dir()
    shutil.rmtree(work, ignore_errors=True)


def test_build_broken_fixture_raises(tmp_path):
    with pytest.raises(subprocess.CalledProcessError):
        build_variant(BROKEN_FIXTURE, VARIANT, repo_root=REPO_ROOT, rejection_log=_rejection_log(tmp_path))


def test_build_broken_fixture_logs_structured_entry(tmp_path):
    log = _rejection_log(tmp_path)
    with pytest.raises(subprocess.CalledProcessError):
        build_variant(BROKEN_FIXTURE, VARIANT, repo_root=REPO_ROOT, rejection_log=log)
    content = log.read_text()
    assert "G1 (build)" in content
    assert EXPECTED_VARIANT_TAG in content


def test_gate_check_good_fixture_passes(tmp_path):
    log = _rejection_log(tmp_path)
    assert run_gates(GOOD_FIXTURE, VARIANT, repo_root=REPO_ROOT, rejection_log=log)


def test_gate_check_good_fixture_produces_no_rejection_entry(tmp_path):
    log = _rejection_log(tmp_path)
    run_gates(GOOD_FIXTURE, VARIANT, repo_root=REPO_ROOT, rejection_log=log)
    assert not log.is_file() or log.read_text().strip() == ""


def test_gate_check_broken_fixture_fails(tmp_path):
    log = _rejection_log(tmp_path)
    assert not run_gates(BROKEN_FIXTURE, VARIANT, repo_root=REPO_ROOT, rejection_log=log)


def test_gate_check_broken_fixture_logs_structured_entry(tmp_path):
    log = _rejection_log(tmp_path)
    run_gates(BROKEN_FIXTURE, VARIANT, repo_root=REPO_ROOT, rejection_log=log)
    content = log.read_text()
    assert "G1 (build)" in content
    assert EXPECTED_VARIANT_TAG in content


def test_gate_check_invalid_ordinal_fails_g2(tmp_path):
    impl_dir = tmp_path / "bad-ordinal" / "impl"
    impl_dir.mkdir(parents=True)
    shutil.copy(GOOD_FIXTURE / "segment.rs", impl_dir / "segment.rs")
    (impl_dir / "manifest.toml").write_text(
        "[variant]\neviction_policy_ordinal = 99\n"
    )
    log = _rejection_log(tmp_path)
    assert not run_gates(impl_dir, VARIANT, repo_root=REPO_ROOT, rejection_log=log)
    content = log.read_text()
    assert "G2 (trait contract)" in content
    assert "99" in content


def test_gate_check_bad_invariant_fixture_fails_g3(tmp_path):
    log = _rejection_log(tmp_path)
    result = run_gates(BAD_INVARIANT_FIXTURE, VARIANT, repo_root=REPO_ROOT, rejection_log=log)
    assert result.g1_passed is True
    assert result.g2_passed is True
    assert result.g3_passed is False
    assert not result.all_passed


def test_gate_check_bad_invariant_fixture_logs_failing_invariant(tmp_path):
    log = _rejection_log(tmp_path)
    run_gates(BAD_INVARIANT_FIXTURE, VARIANT, repo_root=REPO_ROOT, rejection_log=log)
    content = log.read_text()
    assert "G3 (invariant suite)" in content
    assert EXPECTED_VARIANT_TAG in content
    assert "Fingerprint" in content
    # **Failing invariant** must name a P[1-5] property, not just appear in
    # the embedded panic block.
    assert re.search(r"\*\*Failing invariant\*\*: P[1-5] [a-z_]+", content)


def test_gate_check_good_fixture_passes_g3(tmp_path):
    log = _rejection_log(tmp_path)
    result = run_gates(GOOD_FIXTURE, VARIANT, repo_root=REPO_ROOT, rejection_log=log)
    assert result.g1_passed is True
    assert result.g2_passed is True
    assert result.g3_passed is True
    assert result.all_passed
    assert not log.is_file() or log.read_text().strip() == ""
