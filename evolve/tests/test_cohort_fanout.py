"""Cohort fan-out tests for evolve.run_cycle.

Covers:
- cohort.md + hypotheses/HNN.md per Hypothesis
- N variants produced, each with impl/ and eval/
- variant-implementer invoked in parallel (P1 parallelism — in-process threads)
- hypothesis lines from the adapter land in HNN.md and spawn matching variants
- every variant flows through real G1/G2/G3 gates
- failing variant-implementer aborts the cycle with each failure reported

Tests pass small cohort_size (2 or 3) to keep runtime short.
"""

import io
import json
import logging
import re
import shutil
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

import pytest

from evolve import run_cycle
from src.paths import CycleLayout, Layout

from _stubs import StubLLMAdapter

EVOLVE_SRC = Path(__file__).parent.parent
REAL_REPO_ROOT = EVOLVE_SRC.parent

PARALLEL_COHORT_SIZE = 4
IMPLEMENTER_SLEEP_SECONDS = 1
# Parallel total wall time should stay well under the serial lower bound
# (PARALLEL_COHORT_SIZE * sleep). 70% leaves room for fork/exec jitter on
# loaded CI runners while still failing a serial regression.
PARALLEL_WALL_TIME_FRACTION = 0.7


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


def _run_cycle(
    evolve_root: Path,
    *,
    cohort_size: int = 2,
    hypothesis_lines: list[str] | None = None,
    implementer: Callable[..., None] | None = None,
) -> str:
    """Run one cycle in-process. Returns the captured log messages."""

    handler = logging.StreamHandler(io.StringIO())
    handler.setLevel(logging.INFO)
    logging.getLogger().addHandler(handler)
    try:
        adapter = StubLLMAdapter(hypothesis_lines=hypothesis_lines) if hypothesis_lines else StubLLMAdapter(n_lines=cohort_size)
        kwargs = {"implementer": implementer} if implementer is not None else {}
        run_cycle(
            layout=_layout(evolve_root),
            repo_root=REAL_REPO_ROOT,
            cohort_size=cohort_size,
            adapter=adapter,
            **kwargs,
        )
        return handler.stream.getvalue()
    finally:
        logging.getLogger().removeHandler(handler)


@dataclass
class CohortRun:
    cycle_layout: CycleLayout
    log: str


@pytest.fixture(scope="module")
def shared_cohort_run(tmp_path_factory):
    """One cohort_size=3 cycle reused across read-only assertions.

    Each cycle does ~10 cargo invocations (gates + fitness per variant), so
    sharing this fixture cuts the suite by ~3 full cycle runs.
    """

    evolve_root = _setup_repo(tmp_path_factory.mktemp("shared"))
    log = _run_cycle(evolve_root, cohort_size=3)
    return CohortRun(cycle_layout=_layout(evolve_root).cycle("0001"), log=log)


@pytest.mark.slow
def test_cohort_produces_n_variants(shared_cohort_run):
    """A cohort run produces N parallel Variants in one cycle directory."""

    variants_dir = shared_cohort_run.cycle_layout.variants_dir
    variants = sorted(p.name for p in variants_dir.iterdir() if p.is_dir())
    assert variants == ["v001", "v002", "v003"]
    for i in (1, 2, 3):
        variant_layout = shared_cohort_run.cycle_layout.variant(i)
        assert (variant_layout.impl_dir / "segment.rs").is_file()
        assert (variant_layout.impl_dir / "manifest.toml").is_file()
        assert variant_layout.fitness_vector_json.is_file()
        assert variant_layout.insight_md.is_file()


@pytest.mark.slow
def test_cohort_writes_cohort_md_and_per_hypothesis_docs(shared_cohort_run):
    """Orchestrator writes cohort.md and hypotheses/HNN.md per Hypothesis."""

    cohort_md = shared_cohort_run.cycle_layout.cohort_md
    assert cohort_md.is_file()

    hypotheses = shared_cohort_run.cycle_layout.hypotheses_dir
    assert hypotheses.is_dir()
    files = sorted(p.name for p in hypotheses.iterdir() if re.fullmatch(r"H\d{2}\.md", p.name))
    assert files == ["H01.md", "H02.md", "H03.md"]

    body = cohort_md.read_text()
    for hid in ("H01", "H02", "H03"):
        assert hid in body, f"cohort.md must reference {hid}"


@pytest.mark.slow
def test_variant_implementer_invoked_in_parallel(tmp_path):
    """Orchestrator fans out N variant-implementer invocations in parallel (P1).

    Use an overridable implementer that sleeps (releasing the GIL) and
    stamps start+end times; assert total wall time is well below the serial
    lower bound.
    """

    evolve_root = _setup_repo(tmp_path)
    stamp_dir = tmp_path / "stamps"
    stamp_dir.mkdir()

    good_impl = Path(__file__).parent / "fixtures" / "good" / "impl"

    def sleep_implementer(hyp_file, variant_layout, *, layout):
        vid = variant_layout.id
        (stamp_dir / f"{vid}.start").write_text(f"{time.time()}")
        time.sleep(IMPLEMENTER_SLEEP_SECONDS)
        variant_layout.impl_dir.mkdir(parents=True, exist_ok=True)
        shutil.copy(good_impl / "segment.rs", variant_layout.impl_dir / "segment.rs")
        shutil.copy(good_impl / "manifest.toml", variant_layout.impl_dir / "manifest.toml")
        (stamp_dir / f"{vid}.end").write_text(f"{time.time()}")

    _run_cycle(evolve_root, cohort_size=PARALLEL_COHORT_SIZE, implementer=sleep_implementer)

    starts = sorted(float(p.read_text()) for p in stamp_dir.glob("*.start"))
    ends = sorted(float(p.read_text()) for p in stamp_dir.glob("*.end"))
    assert len(starts) == PARALLEL_COHORT_SIZE
    assert len(ends) == PARALLEL_COHORT_SIZE
    total_wall = ends[-1] - starts[0]
    serial_lower_bound = PARALLEL_COHORT_SIZE * IMPLEMENTER_SLEEP_SECONDS
    assert total_wall < serial_lower_bound * PARALLEL_WALL_TIME_FRACTION, (
        f"implementer wall time {total_wall:.2f}s exceeds "
        f"{PARALLEL_WALL_TIME_FRACTION:.0%} of serial bound "
        f"{serial_lower_bound:.2f}s — not parallel"
    )


@pytest.mark.slow
def test_each_variant_flows_through_real_gates(shared_cohort_run):
    """Each Variant flows through the real G1 + G2 + G3 gates."""

    g1 = shared_cohort_run.log.count("G1: passed")
    g2 = shared_cohort_run.log.count("G2: passed")
    g3 = shared_cohort_run.log.count("G3: passed")
    assert g1 == 3, f"expected G1 once per variant, saw {g1}: {shared_cohort_run.log}"
    assert g2 == 3, f"expected G2 once per variant, saw {g2}: {shared_cohort_run.log}"
    assert g3 == 3, f"expected G3 once per variant, saw {g3}: {shared_cohort_run.log}"


@pytest.mark.slow
def test_winners_pareto_front_spans_cohort(shared_cohort_run):
    """All variants evaluated and considered in the Pareto front computation."""

    winners = json.loads(shared_cohort_run.cycle_layout.winners_json.read_text())
    assert len(winners["fitness_vectors"]) == 3
    fv_ids = sorted(fv["variant"] for fv in winners["fitness_vectors"])
    assert fv_ids == ["v001", "v002", "v003"]
    front = winners["pareto_front"]
    assert 1 <= len(front) <= 3
    assert set(front).issubset({"v001", "v002", "v003"})


def test_variant_implementer_agent_markdown_exists():
    """variant-implementer system prompt ships under evolve/src/agents/."""

    agent_md = EVOLVE_SRC / "src" / "agents" / "variant-implementer.md"
    assert agent_md.is_file(), f"missing {agent_md}"
    body = agent_md.read_text()
    assert "hypothesis" in body.lower()
    assert "segment.rs" in body
    assert "manifest.toml" in body
    assert "task.md" in body.lower()


def test_failing_variant_implementer_aborts_cycle(tmp_path, capsys):
    """A failing variant-implementer aborts the cycle with every failure logged."""

    evolve_root = _setup_repo(tmp_path)

    def failing(hyp_file, variant_layout, *, layout):
        raise RuntimeError(f"boom on {hyp_file.name}")

    with pytest.raises(SystemExit):
        _run_cycle(evolve_root, cohort_size=2, implementer=failing)

    captured = capsys.readouterr()
    assert "variant-implementer for v001 failed" in captured.err
    assert "variant-implementer for v002 failed" in captured.err
    assert captured.err.count("boom on") >= 2
    # Cycle never completed → no winners.json.
    assert not _layout(evolve_root).cycle("0001").winners_json.exists()


@pytest.mark.slow
def test_hypothesis_lines_map_to_variants(tmp_path):
    """Each non-empty hypothesis line lands in HNN.md and spawns a Variant."""

    evolve_root = _setup_repo(tmp_path)
    lines = [
        "first hypothesis text",
        "second hypothesis text",
        "third hypothesis text",
    ]
    _run_cycle(evolve_root, cohort_size=3, hypothesis_lines=lines)

    cycle_layout = _layout(evolve_root).cycle("0001")
    files = sorted(
        p.name for p in cycle_layout.hypotheses_dir.iterdir() if re.fullmatch(r"H\d{2}\.md", p.name)
    )
    assert files == ["H01.md", "H02.md", "H03.md"]
    assert "first hypothesis text" in cycle_layout.hypothesis_file(1).read_text()
    assert "second hypothesis text" in cycle_layout.hypothesis_file(2).read_text()
    assert "third hypothesis text" in cycle_layout.hypothesis_file(3).read_text()
    variants = sorted(p.name for p in cycle_layout.variants_dir.iterdir() if p.is_dir())
    assert variants == ["v001", "v002", "v003"]


def test_hypothesis_count_must_match_cohort(tmp_path):
    """Adapter line count must equal cohort_size; mismatch aborts the cycle."""

    evolve_root = _setup_repo(tmp_path)
    lines = ["a", "b", "c"]
    with pytest.raises(ValueError, match="expected 10"):
        _run_cycle(evolve_root, cohort_size=10, hypothesis_lines=lines)


def test_invalid_cohort_size_rejected(capsys):
    """cohort_size must be a positive integer; CLI validation rejects 0/-1."""

    from evolve import main

    for bad in ("0", "-1"):
        with pytest.raises(SystemExit) as exc_info:
            main(["--cohort-size", bad])
        # die() exits 1; argparse would exit 2 — distinguish to avoid vacuous pass.
        assert exc_info.value.code == 1
        assert "--cohort-size" in capsys.readouterr().err
