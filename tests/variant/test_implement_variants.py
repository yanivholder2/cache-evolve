"""Tests for variant implementation inside evolve.run_cycle."""

import io
import json
import logging
import re
import shutil
import time
from dataclasses import dataclass
from pathlib import Path

import pytest

from evolve.cli import run_cycle
from evolve._io import read_utf8, write_utf8
from evolve.config import load_configs
from evolve.identifiers import CycleId, HypothesisId, VariantId
from evolve.paths import CycleLayout, Layout, SourcesLayout

from _stubs import StubLLMAdapter
from _support import configured_subject

REPO_SRC = Path(__file__).parent.parent.parent
SUBJECT = configured_subject(REPO_SRC)

PARALLEL_COHORT_SIZE = 4
IMPLEMENTER_SLEEP_SECONDS = 1
# 70% of serial bound leaves room for fork/exec jitter on loaded CI runners
# while still failing a serial regression.
PARALLEL_WALL_TIME_FRACTION = 0.7


def _setup_repo(tmp_path: Path) -> Path:
    repo_dst = tmp_path / "repo"
    shutil.copytree(
        REPO_SRC,
        repo_dst,
        ignore=shutil.ignore_patterns(
            ".venv", ".git", "target", "cycles", "cycles-ledger.jsonl",
            "__pycache__", "*.pyc", "logs", "outputs", "traces",
        ),
    )
    return repo_dst


def _layout(repo_root: Path) -> Layout:
    return Layout(repo_root=repo_root, outputs_root=repo_root / "outputs", subject=SUBJECT)


def _run_cycle(
    repo_root: Path,
    *,
    cohort_size: int = 2,
    hypothesis_lines: list[str] | None = None,
) -> str:
    """Run one cycle in-process and return the captured log messages."""

    handler = logging.StreamHandler(io.StringIO())
    handler.setLevel(logging.INFO)
    logging.getLogger().addHandler(handler)
    try:
        adapter = StubLLMAdapter(hypothesis_lines=hypothesis_lines) if hypothesis_lines else StubLLMAdapter(n_lines=cohort_size)
        # The stub overlay writes a `.overlay` sentinel, not a manifest impl
        # file; full-autonomy keeps that legal so the pipeline mechanics under
        # test aren't gated by authoring scope.
        cfg = load_configs(repo_root, cohort_size=cohort_size, authoring_scope="full-autonomy")
        run_cycle(
            layout=_layout(repo_root),
            repo_root=repo_root,
            adapter=adapter,
            cfg=cfg,
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
    # Each cycle does ~10 cargo invocations (gates + fitness per variant);
    # sharing the fixture cuts the suite by ~3 full cycle runs.
    repo_root = _setup_repo(tmp_path_factory.mktemp("shared"))
    log = _run_cycle(repo_root, cohort_size=3)
    return CohortRun(cycle_layout=_layout(repo_root).outputs.cycle(CycleId(1)), log=log)


@pytest.mark.slow
def test_cohort_produces_n_variants(shared_cohort_run):
    variants_dir = shared_cohort_run.cycle_layout.variants_dir
    variants = sorted(p.name for p in variants_dir.iterdir() if p.is_dir())
    assert variants == ["v001", "v002", "v003"]
    for i in (1, 2, 3):
        variant_dir = shared_cohort_run.cycle_layout.variant(VariantId(i))
        # Overlay-by-path: every variant ships an `overlay/` directory (under
        # its latest attempt), even when the overlay is empty
        # (baseline-equivalent). The stub adapter writes a `.overlay` sentinel
        # so the dir is non-empty.
        assert variant_dir.overlay_dir.is_dir()
        assert any(variant_dir.overlay_dir.iterdir())
        assert variant_dir.fitness_vector_json.is_file()
        assert variant_dir.insight_md.is_file()


@pytest.mark.slow
def test_cohort_writes_per_hypothesis_docs(shared_cohort_run):
    hypotheses = shared_cohort_run.cycle_layout.hypotheses_dir
    assert hypotheses.is_dir()
    files = sorted(p.name for p in hypotheses.iterdir() if re.fullmatch(r"H\d{2}\.md", p.name))
    assert files == ["H01.md", "H02.md", "H03.md"]


@pytest.mark.slow
def test_variant_implementer_invoked_in_parallel(tmp_path, monkeypatch):
    # Patch in an implementer that sleeps (releasing the GIL) and stamps
    # start+end times; total wall time must stay well below serial bound.
    repo_root = _setup_repo(tmp_path)
    stamp_dir = tmp_path / "stamps"
    stamp_dir.mkdir()

    def sleep_implementer(hyp_file, variant_dir, **_):
        write_utf8(stamp_dir / f"{variant_dir.variant_id}.start", f"{time.time()}")
        time.sleep(IMPLEMENTER_SLEEP_SECONDS)
        # Empty-but-non-vanishing overlay is enough for the cycle to proceed
        # to the gate stage (which the real cargo would then run).
        variant_dir.overlay_dir.mkdir(parents=True, exist_ok=True)
        (variant_dir.overlay_dir / ".overlay").write_text("")
        write_utf8(stamp_dir / f"{variant_dir.variant_id}.end", f"{time.time()}")

    monkeypatch.setattr("evolve.variant.implementer.implement_variant", sleep_implementer)
    _run_cycle(repo_root, cohort_size=PARALLEL_COHORT_SIZE)

    starts = sorted(float(read_utf8(p)) for p in stamp_dir.glob("*.start"))
    ends = sorted(float(read_utf8(p)) for p in stamp_dir.glob("*.end"))
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
    # fitness_vector.json is only written when build gate + contract gate +
    # invariant gate all pass, so a complete vector per variant is the
    # structural proof that every variant cleared the gates.
    expected_dims = {"hit-rate", "per-op-time", "metadata-size"}
    for i in (1, 2, 3):
        variant_dir = shared_cohort_run.cycle_layout.variant(VariantId(i))
        fv = json.loads(read_utf8(variant_dir.fitness_vector_json))
        assert fv["variant"] == f"v{i:03d}"
        dims = {d["dim"] for d in fv["dimensions"]}
        assert dims == expected_dims, f"v{i:03d} dimensions {dims} != {expected_dims}"

    winners = json.loads(read_utf8(shared_cohort_run.cycle_layout.winners_json))
    assert {fv["variant"] for fv in winners["fitness_vectors"]} == {"v001", "v002", "v003"}


@pytest.mark.slow
def test_winners_pareto_front_spans_cohort(shared_cohort_run):
    winners = json.loads(read_utf8(shared_cohort_run.cycle_layout.winners_json))
    assert len(winners["fitness_vectors"]) == 3
    fv_ids = sorted(fv["variant"] for fv in winners["fitness_vectors"])
    assert fv_ids == ["v001", "v002", "v003"]
    front = winners["pareto_front"]
    assert 1 <= len(front) <= 3
    assert set(front).issubset({"v001", "v002", "v003"})


def test_variant_implementer_agent_markdown_exists():
    agent_md = SourcesLayout(root=REPO_SRC).agent_prompt("variant-implementer")
    assert agent_md.is_file(), f"missing {agent_md}"
    body = read_utf8(agent_md)
    lower = body.lower()
    assert "hypothesis" in lower
    assert "overlay/" in body
    assert "factory.rs" in body
    assert "task.md" in lower
    assert "manifest" in lower
    assert "authoring scope" in lower
    assert "extension-points" in body
    assert "full-autonomy" in body


def test_failing_variant_implementer_aborts_cycle(tmp_path, monkeypatch):
    repo_root = _setup_repo(tmp_path)
    invoked: list[str] = []

    def failing(hyp_file, variant_dir, **_):
        invoked.append(hyp_file.name)
        raise RuntimeError(f"boom on {hyp_file.name}")

    monkeypatch.setattr("evolve.variant.implementer.implement_variant", failing)
    with pytest.raises(SystemExit):
        _run_cycle(repo_root, cohort_size=2)

    # Both jobs ran — failures aren't short-circuited.
    assert sorted(invoked) == ["H01.md", "H02.md"]
    # Cycle never completed → no winners.json.
    assert not _layout(repo_root).outputs.cycle(CycleId(1)).winners_json.exists()


@pytest.mark.slow
def test_hypothesis_lines_map_to_variants(tmp_path):
    repo_root = _setup_repo(tmp_path)
    lines = [
        "first hypothesis text",
        "second hypothesis text",
        "third hypothesis text",
    ]
    _run_cycle(repo_root, cohort_size=3, hypothesis_lines=lines)

    cycle_layout = _layout(repo_root).outputs.cycle(CycleId(1))
    files = sorted(
        p.name for p in cycle_layout.hypotheses_dir.iterdir() if re.fullmatch(r"H\d{2}\.md", p.name)
    )
    assert files == ["H01.md", "H02.md", "H03.md"]
    assert "first hypothesis text" in read_utf8(cycle_layout.hypothesis_file(HypothesisId(1)))
    assert "second hypothesis text" in read_utf8(cycle_layout.hypothesis_file(HypothesisId(2)))
    assert "third hypothesis text" in read_utf8(cycle_layout.hypothesis_file(HypothesisId(3)))
    variants = sorted(p.name for p in cycle_layout.variants_dir.iterdir() if p.is_dir())
    assert variants == ["v001", "v002", "v003"]


def test_hypothesis_count_must_match_cohort(tmp_path):
    repo_root = _setup_repo(tmp_path)
    lines = ["a", "b", "c"]
    with pytest.raises(ValueError, match="expected 10"):
        _run_cycle(repo_root, cohort_size=10, hypothesis_lines=lines)


def test_invalid_cohort_size_rejected():
    from evolve.cli import main

    for bad in ("0", "-1"):
        with pytest.raises(SystemExit) as exc_info:
            main(["--cohort-size", bad])
        # die() exits 1, argparse exits 2 — distinguish to avoid vacuous pass.
        assert exc_info.value.code == 1
