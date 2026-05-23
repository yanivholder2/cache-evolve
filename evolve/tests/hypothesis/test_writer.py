"""Tests for evolve/src/hypotheses_writer.write_hypotheses."""

from __future__ import annotations

import re
import shutil
import subprocess
from pathlib import Path

import pytest

from evolve import run_cycle
from src._io import read_utf8, write_utf8
from src.hypothesis.writer import write_hypotheses
from src.identifiers import CycleId, HypothesisId
from src.paths import Layout

from _stubs import StubLLMAdapter


def _layout(evolve: Path) -> Layout:
    return Layout(evolve_root=evolve, outputs_root=evolve / "outputs")

EVOLVE_SRC = Path(__file__).parent.parent.parent
REPO_ROOT = EVOLVE_SRC.parent


def _setup_evolve(tmp_path: Path, *, with_ledger: bool = False, with_leaderboard: bool = False) -> Path:
    # write_hypotheses doesn't read the evolve source tree — only references
    # baseline/extension-points paths in the task.md it writes. Create the
    # outputs the test needs; skip the rest.
    dst = tmp_path / "evolve"
    dst.mkdir()
    layout = _layout(dst)
    layout.outputs.ledger.parent.mkdir(parents=True, exist_ok=True)
    if with_ledger:
        write_utf8(
            layout.outputs.ledger,
            '{"cycle":"0001","cohort_size":2,"pareto_front":["v001"]}\n'
            '{"cycle":"0002","cohort_size":2,"pareto_front":["v001","v002"]}\n',
        )
    else:
        write_utf8(layout.outputs.ledger, "")
    if with_leaderboard:
        write_utf8(
            layout.outputs.leaderboard_md,
            "# Leaderboard\n\n| variant | hit_rate |\n|---|---|\n| v001 | 0.42 |\n",
        )
    return dst


def _copy_real_evolve(tmp_path: Path) -> Path:
    # Full copy is only needed for the e2e slow test that drives run_cycle
    # past task.md into baseline-reading code.
    dst = tmp_path / "evolve"
    shutil.copytree(
        EVOLVE_SRC,
        dst,
        ignore=shutil.ignore_patterns(".venv", "outputs", "__pycache__", "*.pyc"),
    )
    layout = _layout(dst)
    layout.outputs.ledger.parent.mkdir(parents=True, exist_ok=True)
    write_utf8(layout.outputs.ledger, "")
    return dst


def _hypothesis_files(output_dir: Path) -> list[Path]:
    return sorted(p for p in output_dir.iterdir() if re.fullmatch(r"H\d{2}\.md", p.name))


def _stage_prev_cycle(evolve: Path, prev_cycle_id: CycleId) -> None:
    # Continuation cycles require the prev cycle's variants/ dir to exist.
    _layout(evolve).outputs.cycle(prev_cycle_id).variants_dir.mkdir(parents=True, exist_ok=True)


def test_emits_n_distinct_files(tmp_path):
    evolve = _setup_evolve(tmp_path, with_ledger=True, with_leaderboard=True)
    files = write_hypotheses(3, layout=_layout(evolve), cycle_id=CycleId(1), adapter=StubLLMAdapter(n_lines=3))
    assert [p.name for p in files] == ["H01.md", "H02.md", "H03.md"]
    bodies = {read_utf8(p) for p in files}
    assert len(bodies) == 3, "files must be distinct"


def test_too_few_files_fails(tmp_path):
    evolve = _setup_evolve(tmp_path)
    with pytest.raises(ValueError, match="expected 3"):
        write_hypotheses(3, layout=_layout(evolve), cycle_id=CycleId(1), adapter=StubLLMAdapter(n_lines=2))


def test_too_many_files_fails(tmp_path):
    evolve = _setup_evolve(tmp_path)
    with pytest.raises(ValueError, match="expected 3"):
        write_hypotheses(3, layout=_layout(evolve), cycle_id=CycleId(1), adapter=StubLLMAdapter(n_lines=4))


def test_no_files_fails(tmp_path):
    evolve = _setup_evolve(tmp_path)
    with pytest.raises(ValueError):
        write_hypotheses(2, layout=_layout(evolve), cycle_id=CycleId(1), adapter=StubLLMAdapter())


def test_adapter_nonzero_exit_propagates(tmp_path):
    evolve = _setup_evolve(tmp_path)
    with pytest.raises(subprocess.CalledProcessError):
        write_hypotheses(
            3,
            layout=_layout(evolve),
            cycle_id=CycleId(1),
            adapter=StubLLMAdapter(exit_code=5, n_lines=3),
        )


def test_cold_start_with_empty_ledger_and_no_leaderboard(tmp_path):
    evolve = _setup_evolve(tmp_path, with_ledger=False, with_leaderboard=False)
    files = write_hypotheses(2, layout=_layout(evolve), cycle_id=CycleId(1), adapter=StubLLMAdapter(n_lines=2))
    assert len(files) == 2


def test_adapter_invoked_with_expected_argv(tmp_path):
    evolve = _setup_evolve(tmp_path)
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(1),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    body = read_utf8(capture)
    assert "agent: hypotheses-writer" in body
    workspace = _layout(evolve).outputs.cycle(CycleId(1)).hypotheses_dir
    assert f"workspace: {workspace}" in body


def test_hypotheses_writer_passes_outputs_and_subject_as_readable_roots(tmp_path):
    evolve = _setup_evolve(tmp_path)
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(1),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    body = read_utf8(capture)
    layout = _layout(evolve)
    assert str(layout.outputs_root) in body
    assert str(layout.sources.dash_src) in body
    assert str(layout.sources.subject_dir) in body


def test_task_md_written_to_output_dir(tmp_path):
    evolve = _setup_evolve(tmp_path)
    _stage_prev_cycle(evolve, CycleId(6))
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(7),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    body = read_utf8(capture)
    assert "----- task.md -----" in body
    assert "0007" in body
    assert "Number of hypotheses: 2" in body


def test_returns_hypothesis_file_paths(tmp_path):
    evolve = _setup_evolve(tmp_path)
    cycle_layout = _layout(evolve).outputs.cycle(CycleId(1))
    files = write_hypotheses(2, layout=_layout(evolve), cycle_id=CycleId(1), adapter=StubLLMAdapter(n_lines=2))
    assert sorted(files) == [cycle_layout.hypothesis_file(HypothesisId(1)), cycle_layout.hypothesis_file(HypothesisId(2))]


@pytest.mark.slow
def test_cold_cycle_e2e_writes_real_hypothesis_files(tmp_path):
    evolve_root = _copy_real_evolve(tmp_path)
    lines = [
        "Stash-bias eviction lifts hot-set hit rate on Zipfian traces.",
        "Lazy promotion cuts per-op time on read-heavy workloads.",
    ]
    run_cycle(
        layout=_layout(evolve_root),
        repo_root=REPO_ROOT,
        cohort_size=2,
        adapter=StubLLMAdapter(hypothesis_lines=lines),
    )

    cycle_layout = _layout(evolve_root).outputs.cycle(CycleId(1))
    h01 = read_utf8(cycle_layout.hypothesis_file(HypothesisId(1)))
    h02 = read_utf8(cycle_layout.hypothesis_file(HypothesisId(2)))
    assert "Stash-bias eviction" in h01
    assert "Lazy promotion" in h02
    assert "walking-skeleton stub" not in h01
    assert "walking-skeleton stub" not in h02
    assert h01 != h02


def test_cold_start_task_md_grounds_in_baseline(tmp_path):
    evolve = _setup_evolve(tmp_path)
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(1),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    task_md = read_utf8(capture)
    baseline = _layout(evolve).sources.baseline
    # Anchor on "baseline" so a stray "segment.rs" elsewhere doesn't satisfy
    # the check.
    assert "baseline" in task_md
    for path in (baseline.segment_rs, baseline.manifest_toml, baseline.fitness_vector_json, baseline.insight_md):
        assert path.name in task_md, f"{path.name} missing from task.md"


def test_cold_start_task_md_omits_cross_cycle_stubs(tmp_path):
    evolve = _setup_evolve(tmp_path)
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(1),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    task_md = read_utf8(capture)
    layout = _layout(evolve)
    assert str(layout.outputs.leaderboard_md) not in task_md
    assert str(layout.outputs.ledger) not in task_md
    assert str(layout.outputs.synthesis_md) not in task_md


def test_continuation_task_md_grounds_in_previous_cycle(tmp_path):
    evolve = _setup_evolve(tmp_path)
    _stage_prev_cycle(evolve, CycleId(1))
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(2),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    task_md = read_utf8(capture)
    prev = _layout(evolve).outputs.cycle(CycleId(1))
    # Anchor on "0001" so a leaked path from a different cycle wouldn't
    # satisfy the filename checks alone.
    assert "0001" in task_md
    for path in (prev.variants_dir, prev.winners_json, prev.cycle_insight_md):
        assert path.name in task_md, f"{path.name} missing from task.md"


def test_continuation_task_md_lists_baseline_as_valid_parent_variant(tmp_path):
    evolve = _setup_evolve(tmp_path)
    _stage_prev_cycle(evolve, CycleId(1))
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(2),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    task_md = read_utf8(capture)
    assert "baseline" in task_md


def test_continuation_task_md_lists_cross_cycle_artefacts(tmp_path):
    evolve = _setup_evolve(tmp_path)
    _stage_prev_cycle(evolve, CycleId(1))
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(2),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    task_md = read_utf8(capture)
    layout = _layout(evolve)
    assert layout.outputs.leaderboard_md.name in task_md
    assert layout.outputs.ledger.name in task_md
    # synthesis_md basename is generic ("current.md") — anchor on the dir.
    assert "synthesis" in task_md and layout.outputs.synthesis_md.name in task_md


def test_continuation_fails_fast_when_prev_cycle_missing(tmp_path):
    evolve = _setup_evolve(tmp_path)
    with pytest.raises(FileNotFoundError, match="previous cycle"):
        write_hypotheses(
            2,
            layout=_layout(evolve),
            cycle_id=CycleId(2),
            adapter=StubLLMAdapter(n_lines=2),
        )


def test_subagent_markdown_states_contract():
    agent_md = EVOLVE_SRC / "src" / "agents" / "hypotheses-writer.md"
    assert agent_md.is_file(), f"missing {agent_md}"
    body = read_utf8(agent_md)
    assert "hypothesis" in body.lower()
    assert "H<NN>.md" in body
    assert "N distinct" in body or "N Hypotheses" in body
    assert "## Parent" in body
    assert "## Changes" in body
    assert "baseline" in body and "cycle/<NNNN>/v<NNN>" in body
    # Scope limit: agent must not write outside the output dir.
    assert "outside" in body.lower() and "output" in body.lower()
