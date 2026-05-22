"""Tests for the N-cycle loop in evolve.

Tests call run_cycle directly in a loop (parity with evolve.main's outer loop).
"""

import shutil
from pathlib import Path
from typing import Callable

import pytest

from evolve import run_cycle
from paths import Layout

from _stubs import StubLLMAdapter

EVOLVE_SRC = Path(__file__).parent.parent
REAL_REPO_ROOT = EVOLVE_SRC.parent
COHORT = 2


def _setup_repo(tmp_path: Path) -> Path:
    evolve_dst = tmp_path / "evolve"
    shutil.copytree(
        EVOLVE_SRC,
        evolve_dst,
        ignore=shutil.ignore_patterns(".venv", "cycles", "ledger", "__pycache__", "*.pyc", "logs"),
    )
    return evolve_dst


def _layout(evolve_root: Path) -> Layout:
    return Layout(evolve_root=evolve_root, outputs_root=evolve_root / "outputs")


def _run_cycles(evolve_root: Path, n: int, *, implementer: Callable[..., None] | None = None) -> None:
    for _ in range(n):
        kwargs = {"implementer": implementer} if implementer is not None else {}
        run_cycle(
            layout=_layout(evolve_root),
            repo_root=REAL_REPO_ROOT,
            cohort_size=COHORT,
            adapter=StubLLMAdapter(n_lines=COHORT),
            **kwargs,
        )


@pytest.mark.slow
def test_two_cycles_both_sealed(tmp_path):
    evolve_root = _setup_repo(tmp_path)
    _run_cycles(evolve_root, 2)

    layout = _layout(evolve_root)
    for n in ("0001", "0002"):
        cycle_layout = layout.cycle(n)
        assert cycle_layout.root.is_dir(), f"missing cycle {n}"
        assert cycle_layout.sealed_marker.is_file(), f"cycle {n} not sealed"

    rows = layout.ledger.read_text().splitlines()
    assert len(rows) == 2


@pytest.mark.slow
def test_single_cycle_default(tmp_path):
    """One cycle → exactly one sealed directory."""

    evolve_root = _setup_repo(tmp_path)
    _run_cycles(evolve_root, 1)
    cycles_root = _layout(evolve_root).cycles_dir
    cycles = sorted(p.name for p in cycles_root.iterdir() if p.is_dir())
    assert cycles == ["0001"]


def test_loop_aborts_on_cycle_failure(tmp_path):
    """A failing cycle aborts the loop; no later cycle directory appears."""

    evolve_root = _setup_repo(tmp_path)

    def failing(*_args, **_kwargs):
        raise RuntimeError("boom")

    with pytest.raises(SystemExit):
        _run_cycles(evolve_root, 2, implementer=failing)

    assert not _layout(evolve_root).cycle("0002").root.exists()
