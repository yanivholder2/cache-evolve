"""End-to-end test for cycle-start.py.

Runs scripts/cycle-start.py against a temp copy of evolve/ and asserts:
- cycles/0001/ exists with required files
- ledger gains exactly one row
- row has worker_id and no absolute paths
- cycle is sealed (read-only marker present)

Gate cargo is mocked (conftest.py); fitness-plugin cargo runs for real.
A small COHORT_SIZE keeps the per-variant plugin work bounded.
"""

import json
import os
import subprocess
import shutil
from pathlib import Path

EVOLVE_SRC = Path(__file__).parent.parent
SCRIPTS_DIR = EVOLVE_SRC / "scripts"
CYCLE_START = SCRIPTS_DIR / "cycle-start.py"
# Real repo root — fitness plugins need it to find/build Rust binaries
REAL_REPO_ROOT = EVOLVE_SRC.parent


def run_cycle_start(repo_root: Path) -> subprocess.CompletedProcess:
    env = {
        **os.environ,
        "EVOLVE_ROOT": str(repo_root / "evolve"),
        "REPO_ROOT": str(REAL_REPO_ROOT),
        "COHORT_SIZE": "2",
    }
    return subprocess.run(
        [str(CYCLE_START)],
        capture_output=True,
        text=True,
        cwd=repo_root,
        env=env,
    )


def _setup_repo(tmp_path: Path) -> Path:
    """Copy the evolve directory into a tmp repo root so tests don't mutate the real one."""
    evolve_src = Path(__file__).parent.parent
    evolve_dst = tmp_path / "evolve"
    shutil.copytree(evolve_src, evolve_dst, ignore=shutil.ignore_patterns(".venv", "cycles", "ledger", "__pycache__", "*.pyc"))
    return tmp_path


def test_cycle_start_produces_sealed_cycle(tmp_path):
    repo_root = _setup_repo(tmp_path)
    result = run_cycle_start(repo_root)
    assert result.returncode == 0, result.stderr

    cycle_dir = repo_root / "evolve" / "cycles" / "0001"
    assert cycle_dir.is_dir()
    assert (cycle_dir / "cohort.md").is_file()
    assert (cycle_dir / "cycle-insight.md").is_file()
    assert (cycle_dir / "winners.json").is_file()

    variant_dir = cycle_dir / "variants" / "v001"
    assert variant_dir.is_dir()
    assert (variant_dir / "impl").is_dir()
    assert (variant_dir / "eval").is_dir()
    assert (variant_dir / "insight.md").is_file()

    # Sealed
    assert (cycle_dir / "SEALED").is_file()


def test_cycle_start_appends_exactly_one_ledger_row(tmp_path):
    repo_root = _setup_repo(tmp_path)
    result = run_cycle_start(repo_root)
    assert result.returncode == 0, result.stderr

    ledger = repo_root / "evolve" / "ledger" / "cycles.jsonl"
    assert ledger.is_file()
    lines = ledger.read_text().splitlines()
    assert len(lines) == 1


def test_ledger_row_has_worker_id_and_relative_paths(tmp_path):
    repo_root = _setup_repo(tmp_path)
    run_cycle_start(repo_root)

    ledger = repo_root / "evolve" / "ledger" / "cycles.jsonl"
    row = json.loads(ledger.read_text().splitlines()[0])

    assert "worker_id" in row
    assert row["worker_id"] == "local"

    for key, value in row.items():
        if isinstance(value, str):
            assert not value.startswith("/"), f"field '{key}' is absolute: {value!r}"
