"""Tests for scripts/build.py and scripts/gate_check.py.

Cargo is mocked via tests/_mocks/cargo (autouse-wired in conftest.py). The
shim short-circuits build/test calls based on markers embedded in the
variant source, so the gate logic is exercised end-to-end without the cost
of real Rust compilation.
"""

import re
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

from build import build_variant
from gate_check import run_gates

EVOLVE_ROOT = Path(__file__).parent.parent
FIXTURES_DIR = Path(__file__).parent / "fixtures"
REPO_ROOT = EVOLVE_ROOT.parent

GOOD_FIXTURE = FIXTURES_DIR / "good" / "impl"
BROKEN_FIXTURE = FIXTURES_DIR / "broken" / "impl"
BAD_INVARIANT_FIXTURE = FIXTURES_DIR / "bad-invariant" / "impl"


def _rejection_log(tmp_path) -> Path:
    return tmp_path / "rejection-log.md"


def test_build_good_fixture_returns_workspace(tmp_path):
    work = build_variant(GOOD_FIXTURE, "0001", "v001", repo_root=REPO_ROOT, rejection_log=_rejection_log(tmp_path))
    assert work.is_dir()
    shutil.rmtree(work, ignore_errors=True)


def test_build_broken_fixture_raises(tmp_path):
    with pytest.raises(subprocess.CalledProcessError):
        build_variant(BROKEN_FIXTURE, "0001", "v001", repo_root=REPO_ROOT, rejection_log=_rejection_log(tmp_path))


def test_build_broken_fixture_logs_structured_entry(tmp_path):
    log = _rejection_log(tmp_path)
    with pytest.raises(subprocess.CalledProcessError):
        build_variant(BROKEN_FIXTURE, "0001", "v001", repo_root=REPO_ROOT, rejection_log=log)
    content = log.read_text()
    assert "G1 (build)" in content
    assert "cycle=0001" in content
    assert "variant=v001" in content


def test_gate_check_good_fixture_passes(tmp_path):
    log = _rejection_log(tmp_path)
    assert run_gates(GOOD_FIXTURE, "0001", "v001", repo_root=REPO_ROOT, rejection_log=log)


def test_gate_check_good_fixture_produces_no_rejection_entry(tmp_path):
    log = _rejection_log(tmp_path)
    run_gates(GOOD_FIXTURE, "0001", "v001", repo_root=REPO_ROOT, rejection_log=log)
    assert not log.is_file() or log.read_text().strip() == ""


def test_gate_check_broken_fixture_fails(tmp_path):
    log = _rejection_log(tmp_path)
    assert not run_gates(BROKEN_FIXTURE, "0001", "v001", repo_root=REPO_ROOT, rejection_log=log)


def test_gate_check_broken_fixture_logs_structured_entry(tmp_path):
    log = _rejection_log(tmp_path)
    run_gates(BROKEN_FIXTURE, "0001", "v001", repo_root=REPO_ROOT, rejection_log=log)
    content = log.read_text()
    assert "G1 (build)" in content
    assert "cycle=0001" in content
    assert "variant=v001" in content


def test_gate_check_invalid_ordinal_fails_g2(tmp_path):
    """A manifest with eviction_policy_ordinal outside 0–5 fails G2."""

    impl_dir = tmp_path / "bad-ordinal" / "impl"
    impl_dir.mkdir(parents=True)
    shutil.copy(GOOD_FIXTURE / "segment.rs", impl_dir / "segment.rs")
    (impl_dir / "manifest.toml").write_text(
        "[variant]\nname = 'bad'\neviction_policy_ordinal = 99\n"
    )
    log = _rejection_log(tmp_path)
    assert not run_gates(impl_dir, "0001", "v001", repo_root=REPO_ROOT, rejection_log=log)
    content = log.read_text()
    assert "G2 (trait contract)" in content
    assert "99" in content


def test_gate_check_bad_invariant_fixture_fails_g3(tmp_path, caplog):
    """The bad-invariant fixture compiles, passes G2, then fails G3 cleanly."""

    log = _rejection_log(tmp_path)
    with caplog.at_level("INFO"):
        assert not run_gates(BAD_INVARIANT_FIXTURE, "0001", "v001", repo_root=REPO_ROOT, rejection_log=log)
    messages = "\n".join(r.getMessage() for r in caplog.records)
    assert "G2: passed" in messages
    assert "G3: FAILED" in messages
    assert messages.index("G2: passed") < messages.index("G3: FAILED")


def test_gate_check_bad_invariant_fixture_logs_failing_invariant(tmp_path):
    """The G3 rejection entry names the violated invariant + carries an error excerpt."""

    log = _rejection_log(tmp_path)
    run_gates(BAD_INVARIANT_FIXTURE, "0001", "v001", repo_root=REPO_ROOT, rejection_log=log)
    content = log.read_text()
    assert "G3 (invariant suite)" in content
    assert "cycle=0001" in content
    assert "variant=v001" in content
    assert "Fingerprint" in content
    # The extracted **Failing invariant** field must name a P[1-5] property —
    # not just appear somewhere in the embedded panic block.
    assert re.search(r"\*\*Failing invariant\*\*: P[1-5] [a-z_]+", content)


def test_gate_check_g3_failure_stops_before_success_marker(tmp_path, caplog):
    """Gate ordering: a G3-failing variant never reaches the all-gates-passed marker."""

    log = _rejection_log(tmp_path)
    with caplog.at_level("INFO"):
        assert not run_gates(BAD_INVARIANT_FIXTURE, "0001", "v001", repo_root=REPO_ROOT, rejection_log=log)
    messages = "\n".join(r.getMessage() for r in caplog.records)
    assert "all gates passed" not in messages


def test_gate_check_good_fixture_passes_g3(tmp_path, caplog):
    """The good fixture clears G1, G2, and G3 — no rejection entry."""

    log = _rejection_log(tmp_path)
    with caplog.at_level("INFO"):
        assert run_gates(GOOD_FIXTURE, "0001", "v001", repo_root=REPO_ROOT, rejection_log=log)
    messages = "\n".join(r.getMessage() for r in caplog.records)
    assert "G3: passed" in messages
    assert not log.is_file() or log.read_text().strip() == ""
