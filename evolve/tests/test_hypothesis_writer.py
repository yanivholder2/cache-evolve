"""Tests for evolve/scripts/hypothesis_writer.write_hypotheses (Round-2 agentic, file-out).

Covers the contract:
- adapter is invoked with the expected agent_name + output dir
- task.md is written into the output dir before invocation
- exactly N H<NN>.md files written by the adapter → success; wrong count → failure
- adapter raising aborts
- cold-start path still succeeds
- E2E AC: a cold cycle through evolve.run_cycle writes real, distinct HNN.md files
- subagent prompt ships under evolve/scripts/agents/
"""

from __future__ import annotations

import re
import shutil
import subprocess
from pathlib import Path

import pytest

from evolve import run_cycle
from hypothesis_writer import write_hypotheses
from paths import Layout

from _stubs import StubLLMAdapter


def _layout(evolve: Path) -> Layout:
    return Layout(evolve_root=evolve, outputs_root=evolve / "outputs")

EVOLVE_SRC = Path(__file__).parent.parent
REPO_ROOT = EVOLVE_SRC.parent


def _setup_evolve(tmp_path: Path, *, with_ledger: bool = False, with_leaderboard: bool = False) -> Path:
    """Copy a minimal evolve/ into tmp_path and return that evolve_root.

    Outputs (ledger, leaderboard) live under `<evolve_root>/outputs/`.
    """

    dst = tmp_path / "evolve"
    shutil.copytree(
        EVOLVE_SRC,
        dst,
        ignore=shutil.ignore_patterns(".venv", "outputs", "__pycache__", "*.pyc"),
    )
    outputs = dst / "outputs"
    (outputs / "ledger").mkdir(parents=True, exist_ok=True)
    if with_ledger:
        (outputs / "ledger" / "cycles.jsonl").write_text(
            '{"cycle":"0001","cohort_size":2,"pareto_front":["v001"]}\n'
            '{"cycle":"0002","cohort_size":2,"pareto_front":["v001","v002"]}\n',
            encoding="utf-8",
        )
    else:
        (outputs / "ledger" / "cycles.jsonl").write_text("", encoding="utf-8")
    if with_leaderboard:
        (outputs / "leaderboard.md").write_text(
            "# Leaderboard\n\n| variant | hit_rate |\n|---|---|\n| v001 | 0.42 |\n",
            encoding="utf-8",
        )
    return dst


def _hypothesis_files(output_dir: Path) -> list[Path]:
    return sorted(p for p in output_dir.iterdir() if re.fullmatch(r"H\d{2}\.md", p.name))


def test_emits_n_distinct_files(tmp_path):
    """Happy path: exactly N H<NN>.md files in the output dir."""

    evolve = _setup_evolve(tmp_path, with_ledger=True, with_leaderboard=True)
    files = write_hypotheses(3, layout=_layout(evolve), cycle_num="0001", adapter=StubLLMAdapter(n_lines=3))
    assert [p.name for p in files] == ["H01.md", "H02.md", "H03.md"]
    bodies = {p.read_text() for p in files}
    assert len(bodies) == 3, "files must be distinct"


def test_too_few_files_fails(tmp_path):
    """N-1 files from the adapter aborts with a clear error."""

    evolve = _setup_evolve(tmp_path)
    with pytest.raises(ValueError, match="expected 3"):
        write_hypotheses(3, layout=_layout(evolve), cycle_num="0001", adapter=StubLLMAdapter(n_lines=2))


def test_too_many_files_fails(tmp_path):
    """N+1 files aborts (strict)."""

    evolve = _setup_evolve(tmp_path)
    with pytest.raises(ValueError, match="expected 3"):
        write_hypotheses(3, layout=_layout(evolve), cycle_num="0001", adapter=StubLLMAdapter(n_lines=4))


def test_no_files_fails(tmp_path):
    """Adapter writes zero files → wrapper aborts."""

    evolve = _setup_evolve(tmp_path)
    with pytest.raises(ValueError):
        write_hypotheses(2, layout=_layout(evolve), cycle_num="0001", adapter=StubLLMAdapter())


def test_adapter_nonzero_exit_propagates(tmp_path):
    """Adapter raising aborts the cycle."""

    evolve = _setup_evolve(tmp_path)
    with pytest.raises(subprocess.CalledProcessError):
        write_hypotheses(
            3,
            layout=_layout(evolve),
            cycle_num="0001",
            adapter=StubLLMAdapter(exit_code=5, n_lines=3),
        )


def test_cold_start_with_empty_ledger_and_no_leaderboard(tmp_path):
    """Cold start: same code path; empty sections; succeeds."""

    evolve = _setup_evolve(tmp_path, with_ledger=False, with_leaderboard=False)
    files = write_hypotheses(2, layout=_layout(evolve), cycle_num="0001", adapter=StubLLMAdapter(n_lines=2))
    assert len(files) == 2


def test_adapter_invoked_with_expected_argv(tmp_path):
    """Capture the adapter invocation + task.md: agent name + output-dir."""

    evolve = _setup_evolve(tmp_path)
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_num="0001",
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    body = capture.read_text()
    assert "agent: hypothesis-writer" in body
    output_dir = evolve / "outputs" / "cycles" / "0001" / "hypotheses"
    assert f"output-dir: {output_dir}" in body


def test_task_md_written_to_output_dir(tmp_path):
    """task.md exists in the output dir and mentions N + cycle."""

    evolve = _setup_evolve(tmp_path)
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_num="0007",
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    body = capture.read_text()
    assert "----- task.md -----" in body
    assert "0007" in body
    assert "N: 2" in body


def test_returns_hypothesis_file_paths(tmp_path):
    """Function returns discovered file paths so evolve.py can skip re-globbing."""

    evolve = _setup_evolve(tmp_path)
    files = write_hypotheses(2, layout=_layout(evolve), cycle_num="0001", adapter=StubLLMAdapter(n_lines=2))
    output_dir = evolve / "outputs" / "cycles" / "0001" / "hypotheses"
    assert sorted(files) == [output_dir / "H01.md", output_dir / "H02.md"]


@pytest.mark.slow
def test_cold_cycle_e2e_writes_real_hypothesis_files(tmp_path):
    """E2E AC: a cold cycle writes real, distinct HNN.md files from the (stub) LLM."""

    evolve_root = _setup_evolve(tmp_path)
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

    h01 = (evolve_root / "outputs" / "cycles" / "0001" / "hypotheses" / "H01.md").read_text()
    h02 = (evolve_root / "outputs" / "cycles" / "0001" / "hypotheses" / "H02.md").read_text()
    assert "Stash-bias eviction" in h01
    assert "Lazy promotion" in h02
    assert "walking-skeleton stub" not in h01
    assert "walking-skeleton stub" not in h02
    assert h01 != h02


def test_subagent_markdown_exists():
    """hypothesis-writer system prompt ships under evolve/scripts/agents/."""

    agent_md = EVOLVE_SRC / "scripts" / "agents" / "hypothesis-writer.md"
    assert agent_md.is_file(), f"missing {agent_md}"
    body = agent_md.read_text()
    assert "hypothesis" in body.lower()
    assert "H01.md" in body or "H<NN>.md" in body
    assert "task.md" in body.lower()
