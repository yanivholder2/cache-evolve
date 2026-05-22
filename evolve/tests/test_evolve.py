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

import pytest

from evolve import _next_cycle_num, run_cycle
from src.paths import Layout

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


def _layout(evolve_root: Path) -> Layout:
    return Layout(evolve_root=evolve_root, outputs_root=evolve_root / "outputs")


def _run(evolve_root: Path, *, cohort_size: int = 2, adapter=None) -> None:
    run_cycle(
        layout=_layout(evolve_root),
        repo_root=REAL_REPO_ROOT,
        cohort_size=cohort_size,
        adapter=adapter or StubLLMAdapter(n_lines=cohort_size),
    )


@pytest.mark.slow
def test_evolve_produces_sealed_cycle(tmp_path):
    evolve_root = _setup_repo(tmp_path)
    _run(evolve_root)

    cycle_layout = _layout(evolve_root).cycle("0001")
    assert cycle_layout.root.is_dir()
    assert cycle_layout.cohort_md.is_file()
    assert cycle_layout.cycle_insight_md.is_file()
    assert cycle_layout.winners_json.is_file()

    variant_layout = cycle_layout.variant(1)
    assert variant_layout.root.is_dir()
    assert variant_layout.impl_dir.is_dir()
    assert variant_layout.eval_dir.is_dir()
    assert variant_layout.insight_md.is_file()

    assert cycle_layout.sealed_marker.is_file()


@pytest.mark.slow
def test_evolve_appends_exactly_one_ledger_row(tmp_path):
    evolve_root = _setup_repo(tmp_path)
    _run(evolve_root)

    ledger = _layout(evolve_root).ledger
    assert ledger.is_file()
    lines = ledger.read_text().splitlines()
    assert len(lines) == 1


def test_next_cycle_num_empty_dir(tmp_path):
    assert _next_cycle_num(tmp_path / "cycles") == "0001"


def test_next_cycle_num_picks_max_plus_one(tmp_path):
    cycles = tmp_path / "cycles"
    for name in ("0001", "0002", "0005"):
        (cycles / name).mkdir(parents=True)
    assert _next_cycle_num(cycles) == "0006"


def test_next_cycle_num_ignores_non_numeric_entries(tmp_path):
    cycles = tmp_path / "cycles"
    cycles.mkdir()
    (cycles / "0003").mkdir()
    (cycles / "logs").mkdir()
    (cycles / "README.md").write_text("x")
    (cycles / "12345").mkdir()
    assert _next_cycle_num(cycles) == "0004"


@pytest.mark.slow
def test_ledger_row_has_relative_paths(tmp_path):
    evolve_root = _setup_repo(tmp_path)
    _run(evolve_root)

    ledger = _layout(evolve_root).ledger
    row = json.loads(ledger.read_text().splitlines()[0])

    for key, value in row.items():
        if isinstance(value, str):
            assert not value.startswith("/"), f"field '{key}' is absolute: {value!r}"
