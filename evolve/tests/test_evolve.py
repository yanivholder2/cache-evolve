"""End-to-end tests for evolve.run_cycle."""

import json
import shutil
from pathlib import Path

import pytest

from evolve import _next_cycle_id, run_cycle
from src._io import read_utf8, write_utf8
from src.config import load_configs
from src.identifiers import CycleId, VariantId
from src.paths import Layout

from _stubs import StubLLMAdapter

EVOLVE_SRC = Path(__file__).parent.parent
REAL_REPO_ROOT = EVOLVE_SRC.parent


def _setup_repo(tmp_path: Path) -> Path:
    # Copy evolve/ into tmp_path so tests don't mutate the real tree.
    evolve_dst = tmp_path / "evolve"
    shutil.copytree(
        EVOLVE_SRC,
        evolve_dst,
        ignore=shutil.ignore_patterns(
            ".venv",
            "cycles",
            "ledger",
            "logs",
            "cross-cycle-insights.md",
            "leaderboard.md",
            "__pycache__",
            "*.pyc",
        ),
    )
    return evolve_dst


def _layout(evolve_root: Path) -> Layout:
    return Layout(evolve_root=evolve_root, outputs_root=evolve_root / "outputs")


def _run(evolve_root: Path, *, cohort_size: int = 2, adapter=None) -> None:
    cfg = load_configs(evolve_root, cohort_size=cohort_size)
    run_cycle(
        layout=_layout(evolve_root),
        repo_root=REAL_REPO_ROOT,
        adapter=adapter or StubLLMAdapter(n_lines=cohort_size),
        cfg=cfg,
    )


@pytest.mark.slow
def test_evolve_produces_sealed_cycle(tmp_path):
    evolve_root = _setup_repo(tmp_path)
    _run(evolve_root)

    cycle_layout = _layout(evolve_root).outputs.cycle(CycleId(1))
    assert cycle_layout.root.is_dir()
    assert cycle_layout.cycle_insight_md.is_file()
    assert cycle_layout.winners_json.is_file()

    variant_dir = cycle_layout.variant(VariantId(1))
    assert variant_dir.root.is_dir()
    assert variant_dir.impl_dir.is_dir()
    assert variant_dir.eval_dir.is_dir()
    assert variant_dir.insight_md.is_file()

    assert cycle_layout.sealed_marker.is_file()


@pytest.mark.slow
def test_evolve_appends_exactly_one_ledger_row(tmp_path):
    evolve_root = _setup_repo(tmp_path)
    _run(evolve_root)

    ledger = _layout(evolve_root).outputs.ledger
    assert ledger.is_file()
    lines = read_utf8(ledger).splitlines()
    assert len(lines) == 1


def test_next_cycle_id_empty_dir(tmp_path):
    assert _next_cycle_id(tmp_path / "cycles") == CycleId(1)


def test_next_cycle_id_picks_max_plus_one(tmp_path):
    cycles = tmp_path / "cycles"
    for name in ("0001", "0002", "0005"):
        (cycles / name).mkdir(parents=True)
    assert _next_cycle_id(cycles) == CycleId(6)


def test_next_cycle_id_ignores_non_numeric_entries(tmp_path):
    cycles = tmp_path / "cycles"
    cycles.mkdir()
    (cycles / "0003").mkdir()
    (cycles / "logs").mkdir()
    write_utf8(cycles / "README.md", "x")
    (cycles / "12345").mkdir()
    assert _next_cycle_id(cycles) == CycleId(4)


@pytest.mark.slow
def test_ledger_row_has_relative_paths(tmp_path):
    evolve_root = _setup_repo(tmp_path)
    _run(evolve_root)

    ledger = _layout(evolve_root).outputs.ledger
    row = json.loads(read_utf8(ledger).splitlines()[0])

    for key, value in row.items():
        if isinstance(value, str):
            assert not value.startswith("/"), f"field '{key}' is absolute: {value!r}"


class _CrossCycleFailingAdapter(StubLLMAdapter):
    """StubLLMAdapter that raises only for the cross-cycle-insight-writer agent."""

    def run(self, agent_name, workspace, *, readable_roots, allow_edits):
        if agent_name == "cross-cycle-insight-writer":
            raise RuntimeError("simulated cross-cycle writer failure")
        return super().run(agent_name, workspace, readable_roots=readable_roots, allow_edits=allow_edits)


@pytest.mark.slow
def test_cross_cycle_writer_failure_does_not_block_seal(tmp_path):
    evolve_root = _setup_repo(tmp_path)
    _run(evolve_root, adapter=_CrossCycleFailingAdapter(n_lines=2))

    cycle_layout = _layout(evolve_root).outputs.cycle(CycleId(1))
    assert cycle_layout.sealed_marker.is_file(), "seal must complete despite cross-cycle writer failure"
    assert _layout(evolve_root).outputs.ledger.is_file()
    assert not _layout(evolve_root).outputs.cross_cycle_insights_md.exists(), (
        "no prior file existed, so the failed writer must leave nothing at the destination"
    )
