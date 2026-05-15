"""Integration tests for scripts/build.sh and scripts/gate-check.sh.

These tests invoke real Rust compilation (~30s each). They require cargo on PATH.
"""

import os
import shutil
import subprocess
from pathlib import Path

import pytest

EVOLVE_ROOT = Path(__file__).parent.parent
SCRIPTS_DIR = EVOLVE_ROOT / "scripts"
FIXTURES_DIR = EVOLVE_ROOT / "fixtures"
REPO_ROOT = EVOLVE_ROOT.parent

BUILD_SCRIPT = SCRIPTS_DIR / "build.sh"
GATE_CHECK_SCRIPT = SCRIPTS_DIR / "gate-check.sh"

GOOD_FIXTURE = FIXTURES_DIR / "good" / "impl"
BROKEN_FIXTURE = FIXTURES_DIR / "broken" / "impl"

if not shutil.which("cargo"):
    pytest.skip("cargo not on PATH", allow_module_level=True)


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
