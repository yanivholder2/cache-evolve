"""Tests for scripts/build.sh and scripts/gate-check.sh.

Cargo is mocked via tests/_mocks/cargo (autouse-wired in conftest.py). The
shim short-circuits build/test calls based on markers embedded in the
variant source, so the bash gate logic is exercised end-to-end without
the cost of real Rust compilation.
"""

import os
import re
import shutil
import subprocess
from pathlib import Path

EVOLVE_ROOT = Path(__file__).parent.parent
SCRIPTS_DIR = EVOLVE_ROOT / "scripts"
FIXTURES_DIR = EVOLVE_ROOT / "fixtures"
REPO_ROOT = EVOLVE_ROOT.parent

BUILD_SCRIPT = SCRIPTS_DIR / "build.sh"
GATE_CHECK_SCRIPT = SCRIPTS_DIR / "gate-check.sh"

GOOD_FIXTURE = FIXTURES_DIR / "good" / "impl"
BROKEN_FIXTURE = FIXTURES_DIR / "broken" / "impl"
BAD_INVARIANT_FIXTURE = FIXTURES_DIR / "bad-invariant" / "impl"


def _run(script, args, tmp_path):
    rejection_log = tmp_path / "rejection-log.md"
    env = {
        **os.environ,
        "EVOLVE_ROOT": str(tmp_path),
        "REPO_ROOT": str(REPO_ROOT),
        "REJECTION_LOG": str(rejection_log),
    }
    result = subprocess.run(
        [str(script)] + [str(a) for a in args],
        capture_output=True,
        text=True,
        env=env,
    )
    return result, rejection_log


def test_build_good_fixture_exits_zero(tmp_path):
    result, _ = _run(BUILD_SCRIPT, [GOOD_FIXTURE, "0001", "v001"], tmp_path)
    assert result.returncode == 0, result.stderr
    workspace = Path(result.stdout.strip())
    assert workspace.is_dir()
    shutil.rmtree(workspace, ignore_errors=True)


def test_build_broken_fixture_exits_nonzero(tmp_path):
    result, log = _run(BUILD_SCRIPT, [BROKEN_FIXTURE, "0001", "v001"], tmp_path)
    assert result.returncode != 0


def test_build_broken_fixture_logs_structured_entry(tmp_path):
    _run(BUILD_SCRIPT, [BROKEN_FIXTURE, "0001", "v001"], tmp_path)
    log = tmp_path / "rejection-log.md"
    assert log.is_file()
    content = log.read_text()
    assert "G1 (build)" in content
    assert "cycle=0001" in content
    assert "variant=v001" in content


def test_gate_check_good_fixture_exits_zero(tmp_path):
    result, _ = _run(GATE_CHECK_SCRIPT, [GOOD_FIXTURE, "0001", "v001"], tmp_path)
    assert result.returncode == 0, result.stderr


def test_gate_check_good_fixture_produces_no_rejection_entry(tmp_path):
    _, log = _run(GATE_CHECK_SCRIPT, [GOOD_FIXTURE, "0001", "v001"], tmp_path)
    assert not log.is_file() or log.read_text().strip() == ""


def test_gate_check_broken_fixture_exits_nonzero(tmp_path):
    result, _ = _run(GATE_CHECK_SCRIPT, [BROKEN_FIXTURE, "0001", "v001"], tmp_path)
    assert result.returncode != 0


def test_gate_check_broken_fixture_logs_structured_entry(tmp_path):
    _, log = _run(GATE_CHECK_SCRIPT, [BROKEN_FIXTURE, "0001", "v001"], tmp_path)
    assert log.is_file()
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
    result, log = _run(GATE_CHECK_SCRIPT, [impl_dir, "0001", "v001"], tmp_path)
    assert result.returncode != 0
    content = log.read_text()
    assert "G2 (trait contract)" in content
    assert "99" in content


def test_gate_check_bad_invariant_fixture_fails_g3(tmp_path):
    """The bad-invariant fixture compiles, passes G2, then fails G3 cleanly."""
    result, _ = _run(GATE_CHECK_SCRIPT, [BAD_INVARIANT_FIXTURE, "0001", "v001"], tmp_path)
    assert result.returncode != 0
    assert "G2: passed" in result.stderr
    assert "G3: FAILED" in result.stderr
    assert result.stderr.index("G2: passed") < result.stderr.index("G3: FAILED")


def test_gate_check_bad_invariant_fixture_logs_failing_invariant(tmp_path):
    """The G3 rejection entry names the violated invariant + carries an error excerpt."""
    _, log = _run(GATE_CHECK_SCRIPT, [BAD_INVARIANT_FIXTURE, "0001", "v001"], tmp_path)
    assert log.is_file()
    content = log.read_text()
    assert "G3 (invariant suite)" in content
    assert "cycle=0001" in content
    assert "variant=v001" in content
    assert "Fingerprint" in content
    # The extracted **Failing invariant** field must name a P[1-5] property —
    # not just appear somewhere in the embedded panic block.
    assert re.search(r"\*\*Failing invariant\*\*: P[1-5] [a-z_]+", content)


def test_gate_check_g3_failure_stops_before_success_marker(tmp_path):
    """Gate ordering: a G3-failing variant never reaches the all-gates-passed marker,
    so any downstream fitness step keyed off gate-check's success exit is skipped."""
    result, _ = _run(GATE_CHECK_SCRIPT, [BAD_INVARIANT_FIXTURE, "0001", "v001"], tmp_path)
    assert result.returncode != 0
    assert "all gates passed" not in result.stderr


def test_gate_check_good_fixture_passes_g3(tmp_path):
    """The good fixture clears G1, G2, and G3 — no rejection entry."""
    result, log = _run(GATE_CHECK_SCRIPT, [GOOD_FIXTURE, "0001", "v001"], tmp_path)
    assert result.returncode == 0, result.stderr
    assert "G3: passed" in result.stderr
    assert not log.is_file() or log.read_text().strip() == ""
