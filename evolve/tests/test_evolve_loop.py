"""Tests for the N-cycle loop in evolve.py.

EVOLVE_CYCLES drives the outer loop; default is one cycle (parity with the
pre-loop behaviour). Aborts on the first cycle failure.
"""

import os
import shutil
import subprocess
from pathlib import Path

EVOLVE_SRC = Path(__file__).parent.parent
SCRIPTS_DIR = EVOLVE_SRC / "scripts"
EVOLVE_PY = SCRIPTS_DIR / "evolve.py"
LLM_STUB = EVOLVE_SRC / "tests" / "_mocks" / "llm-stub.sh"
REAL_REPO_ROOT = EVOLVE_SRC.parent


def _setup_repo(tmp_path: Path) -> Path:
    evolve_dst = tmp_path / "evolve"
    shutil.copytree(
        EVOLVE_SRC,
        evolve_dst,
        ignore=shutil.ignore_patterns(".venv", "cycles", "ledger", "__pycache__", "*.pyc"),
    )
    return tmp_path


def _run(repo_root: Path, extra_env: dict[str, str] | None = None) -> subprocess.CompletedProcess:
    env = {
        **os.environ,
        "EVOLVE_ROOT": str(repo_root / "evolve"),
        "REPO_ROOT": str(REAL_REPO_ROOT),
        "COHORT_SIZE": "2",
        "EVOLVE_LLM_CMD": str(LLM_STUB),
        "STUB_LINES": "2",
    }
    if extra_env:
        env.update(extra_env)
    return subprocess.run(
        [str(EVOLVE_PY)],
        capture_output=True,
        text=True,
        cwd=repo_root,
        env=env,
    )


def test_evolve_cycles_env_runs_n_cycles(tmp_path):
    """EVOLVE_CYCLES=2 → cycles 0001 and 0002 both sealed; ledger has 2 rows."""
    repo_root = _setup_repo(tmp_path)
    result = _run(repo_root, {"EVOLVE_CYCLES": "2"})
    assert result.returncode == 0, result.stderr

    for n in ("0001", "0002"):
        cycle_dir = repo_root / "evolve" / "cycles" / n
        assert cycle_dir.is_dir(), f"missing cycle {n}"
        assert (cycle_dir / "SEALED").is_file(), f"cycle {n} not sealed"

    ledger = repo_root / "evolve" / "ledger" / "cycles.jsonl"
    assert ledger.is_file()
    rows = ledger.read_text().splitlines()
    assert len(rows) == 2


def test_evolve_default_is_single_cycle(tmp_path):
    """No EVOLVE_CYCLES → exactly one cycle (parity with the prior behaviour)."""
    repo_root = _setup_repo(tmp_path)
    result = _run(repo_root)
    assert result.returncode == 0, result.stderr

    cycles_root = repo_root / "evolve" / "cycles"
    cycles = sorted(p.name for p in cycles_root.iterdir() if p.is_dir())
    assert cycles == ["0001"]


def test_evolve_loop_aborts_on_cycle_failure(tmp_path):
    """A failing cycle aborts the loop; no later cycle directory appears."""
    repo_root = _setup_repo(tmp_path)
    failing = tmp_path / "fail-implementer.sh"
    failing.write_text(
        "#!/usr/bin/env bash\n"
        'echo "boom" >&2\n'
        "exit 7\n"
    )
    failing.chmod(0o755)
    result = _run(
        repo_root,
        {"EVOLVE_CYCLES": "2", "VARIANT_IMPLEMENTER": str(failing)},
    )
    assert result.returncode != 0
    assert not (repo_root / "evolve" / "cycles" / "0002").exists()


def test_invalid_evolve_cycles_rejected(tmp_path):
    """EVOLVE_CYCLES must be a positive integer."""
    repo_root = _setup_repo(tmp_path)
    for bad in ("0", "abc", "-1"):
        result = _run(repo_root, {"EVOLVE_CYCLES": bad})
        assert result.returncode != 0, f"expected failure for EVOLVE_CYCLES={bad!r}"
        assert "EVOLVE_CYCLES" in result.stderr
    assert not (repo_root / "evolve" / "cycles" / "0001").exists()
