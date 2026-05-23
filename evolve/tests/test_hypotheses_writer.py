"""Tests for evolve/src/hypotheses_writer.write_hypotheses (Round-2 agentic, file-out).

Covers the contract:
- adapter is invoked with the expected agent_name + output dir
- task.md is written into the output dir before invocation
- exactly N H<NN>.md files written by the adapter → success; wrong count → failure
- adapter raising aborts
- cold-start path still succeeds
- E2E AC: a cold cycle through evolve.run_cycle writes real, distinct HNN.md files
- subagent prompt ships under evolve/src/agents/
"""

from __future__ import annotations

import re
import shutil
import subprocess
from pathlib import Path

import pytest

from evolve import run_cycle
from src.hypotheses_writer import write_hypotheses
from src.identifiers import CycleId, HypothesisId
from src.paths import Layout

from _stubs import StubLLMAdapter


def _layout(evolve: Path) -> Layout:
    return Layout(evolve_root=evolve, outputs_root=evolve / "outputs")

EVOLVE_SRC = Path(__file__).parent.parent
REPO_ROOT = EVOLVE_SRC.parent


def _setup_evolve(tmp_path: Path, *, with_ledger: bool = False, with_leaderboard: bool = False) -> Path:
    """Build a minimal evolve layout under `tmp_path/evolve/`.

    `write_hypotheses` doesn't read the evolve source tree — it just
    references baseline/extension-points paths in the task.md it writes.
    Outputs (ledger, leaderboard) live under `<evolve_root>/outputs/`;
    create the ones the test needs and skip the rest of the tree.
    """

    dst = tmp_path / "evolve"
    dst.mkdir()
    layout = _layout(dst)
    layout.ledger.parent.mkdir(parents=True, exist_ok=True)
    if with_ledger:
        layout.ledger.write_text(
            '{"cycle":"0001","cohort_size":2,"pareto_front":["v001"]}\n'
            '{"cycle":"0002","cohort_size":2,"pareto_front":["v001","v002"]}\n',
            encoding="utf-8",
        )
    else:
        layout.ledger.write_text("", encoding="utf-8")
    if with_leaderboard:
        layout.leaderboard_md.write_text(
            "# Leaderboard\n\n| variant | hit_rate |\n|---|---|\n| v001 | 0.42 |\n",
            encoding="utf-8",
        )
    return dst


def _copy_real_evolve(tmp_path: Path) -> Path:
    """Full copy of the real evolve tree — only needed for the e2e slow
    test that drives `run_cycle` past task.md into baseline-reading code."""

    dst = tmp_path / "evolve"
    shutil.copytree(
        EVOLVE_SRC,
        dst,
        ignore=shutil.ignore_patterns(".venv", "outputs", "__pycache__", "*.pyc"),
    )
    layout = _layout(dst)
    layout.ledger.parent.mkdir(parents=True, exist_ok=True)
    layout.ledger.write_text("", encoding="utf-8")
    return dst


def _hypothesis_files(output_dir: Path) -> list[Path]:
    return sorted(p for p in output_dir.iterdir() if re.fullmatch(r"H\d{2}\.md", p.name))


def _stage_prev_cycle(evolve: Path, prev_cycle_id: CycleId) -> None:
    """Continuation cycles require the prev cycle's variants/ dir to exist."""
    _layout(evolve).cycle(prev_cycle_id).variants_dir.mkdir(parents=True, exist_ok=True)


def test_emits_n_distinct_files(tmp_path):
    """Happy path: exactly N H<NN>.md files in the output dir."""

    evolve = _setup_evolve(tmp_path, with_ledger=True, with_leaderboard=True)
    files = write_hypotheses(3, layout=_layout(evolve), cycle_id=CycleId(1), adapter=StubLLMAdapter(n_lines=3))
    assert [p.name for p in files] == ["H01.md", "H02.md", "H03.md"]
    bodies = {p.read_text() for p in files}
    assert len(bodies) == 3, "files must be distinct"


def test_too_few_files_fails(tmp_path):
    """N-1 files from the adapter aborts with a clear error."""

    evolve = _setup_evolve(tmp_path)
    with pytest.raises(ValueError, match="expected 3"):
        write_hypotheses(3, layout=_layout(evolve), cycle_id=CycleId(1), adapter=StubLLMAdapter(n_lines=2))


def test_too_many_files_fails(tmp_path):
    """N+1 files aborts (strict)."""

    evolve = _setup_evolve(tmp_path)
    with pytest.raises(ValueError, match="expected 3"):
        write_hypotheses(3, layout=_layout(evolve), cycle_id=CycleId(1), adapter=StubLLMAdapter(n_lines=4))


def test_no_files_fails(tmp_path):
    """Adapter writes zero files → wrapper aborts."""

    evolve = _setup_evolve(tmp_path)
    with pytest.raises(ValueError):
        write_hypotheses(2, layout=_layout(evolve), cycle_id=CycleId(1), adapter=StubLLMAdapter())


def test_adapter_nonzero_exit_propagates(tmp_path):
    """Adapter raising aborts the cycle."""

    evolve = _setup_evolve(tmp_path)
    with pytest.raises(subprocess.CalledProcessError):
        write_hypotheses(
            3,
            layout=_layout(evolve),
            cycle_id=CycleId(1),
            adapter=StubLLMAdapter(exit_code=5, n_lines=3),
        )


def test_cold_start_with_empty_ledger_and_no_leaderboard(tmp_path):
    """Cold start: same code path; empty sections; succeeds."""

    evolve = _setup_evolve(tmp_path, with_ledger=False, with_leaderboard=False)
    files = write_hypotheses(2, layout=_layout(evolve), cycle_id=CycleId(1), adapter=StubLLMAdapter(n_lines=2))
    assert len(files) == 2


def test_adapter_invoked_with_expected_argv(tmp_path):
    """Capture the adapter invocation + task.md: agent name + output-dir."""

    evolve = _setup_evolve(tmp_path)
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(1),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    body = capture.read_text()
    assert "agent: hypotheses-writer" in body
    workspace = _layout(evolve).cycle(CycleId(1)).hypotheses_dir
    assert f"workspace: {workspace}" in body


def test_hypotheses_writer_passes_outputs_and_subject_as_readable_roots(tmp_path):
    """write_hypotheses must expose outputs_root, dash_src, and subject_dir
    so the agent can read prior cycles, the dash crate, and baseline."""

    evolve = _setup_evolve(tmp_path)
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(1),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    body = capture.read_text()
    layout = _layout(evolve)
    assert str(layout.outputs_root) in body
    assert str(layout.dash_src) in body
    assert str(layout.subject_dir) in body


def test_task_md_written_to_output_dir(tmp_path):
    """task.md exists in the output dir and mentions N + cycle."""

    evolve = _setup_evolve(tmp_path)
    _stage_prev_cycle(evolve, CycleId(6))
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(7),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    body = capture.read_text()
    assert "----- task.md -----" in body
    assert "0007" in body
    assert "Number of hypotheses: 2" in body


def test_returns_hypothesis_file_paths(tmp_path):
    """Function returns discovered file paths so evolve.py can skip re-globbing."""

    evolve = _setup_evolve(tmp_path)
    cycle_layout = _layout(evolve).cycle(CycleId(1))
    files = write_hypotheses(2, layout=_layout(evolve), cycle_id=CycleId(1), adapter=StubLLMAdapter(n_lines=2))
    assert sorted(files) == [cycle_layout.hypothesis_file(HypothesisId(1)), cycle_layout.hypothesis_file(HypothesisId(2))]


@pytest.mark.slow
def test_cold_cycle_e2e_writes_real_hypothesis_files(tmp_path):
    """E2E AC: a cold cycle writes real, distinct HNN.md files from the (stub) LLM."""

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

    cycle_layout = _layout(evolve_root).cycle(CycleId(1))
    h01 = cycle_layout.hypothesis_file(HypothesisId(1)).read_text()
    h02 = cycle_layout.hypothesis_file(HypothesisId(2)).read_text()
    assert "Stash-bias eviction" in h01
    assert "Lazy promotion" in h02
    assert "walking-skeleton stub" not in h01
    assert "walking-skeleton stub" not in h02
    assert h01 != h02


def test_cold_start_task_md_grounds_in_baseline(tmp_path):
    """Cycle 0001 task.md points the writer at every baseline file."""

    evolve = _setup_evolve(tmp_path)
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(1),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    task_md = capture.read_text()
    baseline = _layout(evolve).baseline
    # Anchor on the "baseline" segment so a stray "segment.rs" elsewhere
    # doesn't satisfy the check; assert each filename appears near it.
    assert "baseline" in task_md
    for path in (baseline.segment_rs, baseline.manifest_toml, baseline.fitness_vector_json, baseline.insight_md):
        assert path.name in task_md, f"{path.name} missing from task.md"


def test_cold_start_task_md_omits_cross_cycle_stubs(tmp_path):
    """Cycle 0001 has no history; task.md does not list leaderboard/ledger/synthesis."""

    evolve = _setup_evolve(tmp_path)
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(1),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    task_md = capture.read_text()
    layout = _layout(evolve)
    assert str(layout.leaderboard_md) not in task_md
    assert str(layout.ledger) not in task_md
    assert str(layout.synthesis_md) not in task_md


def test_continuation_task_md_grounds_in_previous_cycle(tmp_path):
    """Cycle 0002+ task.md points the writer at the previous cycle's artefacts."""

    evolve = _setup_evolve(tmp_path)
    _stage_prev_cycle(evolve, CycleId(1))
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(2),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    task_md = capture.read_text()
    prev = _layout(evolve).cycle(CycleId(1))
    # `0001/` anchors the references to the prev cycle so a leaked path
    # from a different cycle wouldn't satisfy the filename checks alone.
    assert "0001" in task_md
    for path in (prev.variants_dir, prev.winners_json, prev.cycle_insight_md):
        assert path.name in task_md, f"{path.name} missing from task.md"


def test_continuation_task_md_lists_baseline_as_valid_parent_variant(tmp_path):
    """Baseline is always a valid parent variant — continuation task.md names it."""

    evolve = _setup_evolve(tmp_path)
    _stage_prev_cycle(evolve, CycleId(1))
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(2),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    task_md = capture.read_text()
    assert "baseline" in task_md


def test_continuation_task_md_lists_cross_cycle_artefacts(tmp_path):
    """Cycle 0002+ task.md includes leaderboard, ledger, and synthesis paths."""

    evolve = _setup_evolve(tmp_path)
    _stage_prev_cycle(evolve, CycleId(1))
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(2),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    task_md = capture.read_text()
    layout = _layout(evolve)
    assert layout.leaderboard_md.name in task_md  # leaderboard.md
    assert layout.ledger.name in task_md          # cycles.jsonl
    # synthesis_md basename is generic ("current.md"); anchor on the dir.
    assert "synthesis" in task_md and layout.synthesis_md.name in task_md


def test_continuation_fails_fast_when_prev_cycle_missing(tmp_path):
    """Cycle 0002+ without cycles/0001/variants/ on disk must fail fast,
    not emit task.md pointing at nonexistent paths."""

    evolve = _setup_evolve(tmp_path)
    with pytest.raises(FileNotFoundError, match="previous cycle"):
        write_hypotheses(
            2,
            layout=_layout(evolve),
            cycle_id=CycleId(2),
            adapter=StubLLMAdapter(n_lines=2),
        )


def test_subagent_markdown_states_contract():
    """The hypotheses-writer system prompt ships the contract the framework
    depends on: exactly N H<NN>.md files, baseline-or-past-variant grounding,
    and a workspace scope limit so the agent doesn't escape the cohort dir.
    """

    agent_md = EVOLVE_SRC / "src" / "agents" / "hypotheses-writer.md"
    assert agent_md.is_file(), f"missing {agent_md}"
    body = agent_md.read_text()
    assert "hypothesis" in body.lower()
    # Output count + naming pattern: exactly N files named H<NN>.md.
    assert "H<NN>.md" in body
    assert "N distinct" in body or "N Hypotheses" in body
    # Required sections downstream parsers rely on.
    assert "## Parent" in body
    assert "## Changes" in body
    # Grounding: parent variant must be baseline or a past variant.
    assert "baseline" in body and "cycle/<NNNN>/v<NNN>" in body
    # Scope limit: must not write outside the output dir.
    assert "outside" in body.lower() and "output" in body.lower()
