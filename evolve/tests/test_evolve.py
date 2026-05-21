"""End-to-end test for evolve.run_cycle.

Runs run_cycle against a temp copy of evolve/ and asserts:
- cycles/0001/ exists with required files
- ledger gains exactly one row
- row has no absolute paths
- cycle is sealed (read-only marker present)

Gate cargo is mocked (conftest.py); fitness-plugin cargo runs for real.
A small cohort_size keeps the per-variant plugin work bounded.
"""

import json
import shutil
from pathlib import Path

from evolve import run_cycle

from _stubs import StubLLMAdapter

EVOLVE_SRC = Path(__file__).parent.parent
REAL_REPO_ROOT = EVOLVE_SRC.parent


def _setup_repo(tmp_path: Path) -> Path:
    """Copy evolve/ into tmp_path so tests don't mutate the real tree."""

    evolve_dst = tmp_path / "evolve"
    shutil.copytree(
        EVOLVE_SRC,
        evolve_dst,
        ignore=shutil.ignore_patterns(".venv", "cycles", "ledger", "__pycache__", "*.pyc", "logs"),
    )
    return evolve_dst


def _run(evolve_root: Path, *, cohort_size: int = 2, adapter=None) -> None:
    run_cycle(
        evolve_root=evolve_root,
        repo_root=REAL_REPO_ROOT,
        cohort_size=cohort_size,
        adapter=adapter or StubLLMAdapter(n_lines=cohort_size),
        outputs_root=evolve_root / "outputs",
    )


def test_evolve_produces_sealed_cycle(tmp_path):
    evolve_root = _setup_repo(tmp_path)
    _run(evolve_root)

    cycle_dir = evolve_root / "outputs" / "cycles" / "0001"
    assert cycle_dir.is_dir()
    assert (cycle_dir / "cohort.md").is_file()
    assert (cycle_dir / "cycle-insight.md").is_file()
    assert (cycle_dir / "winners.json").is_file()

    variant_dir = cycle_dir / "variants" / "v001"
    assert variant_dir.is_dir()
    assert (variant_dir / "impl").is_dir()
    assert (variant_dir / "eval").is_dir()
    assert (variant_dir / "insight.md").is_file()

    assert (cycle_dir / "SEALED").is_file()


def test_evolve_appends_exactly_one_ledger_row(tmp_path):
    evolve_root = _setup_repo(tmp_path)
    _run(evolve_root)

    ledger = evolve_root / "outputs" / "ledger" / "cycles.jsonl"
    assert ledger.is_file()
    lines = ledger.read_text().splitlines()
    assert len(lines) == 1


def test_ledger_row_has_relative_paths(tmp_path):
    evolve_root = _setup_repo(tmp_path)
    _run(evolve_root)

    ledger = evolve_root / "outputs" / "ledger" / "cycles.jsonl"
    row = json.loads(ledger.read_text().splitlines()[0])

    for key, value in row.items():
        if isinstance(value, str):
            assert not value.startswith("/"), f"field '{key}' is absolute: {value!r}"
