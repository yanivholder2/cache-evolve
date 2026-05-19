"""Tests for evolve/scripts/hypothesis-writer.py (Round-2 agentic, file-out).

Covers the contract:
- adapter argv has the expected shape (<agent-name>, <output-dir>, --writable ...)
- task.md is written into the output dir before invocation
- exactly N H<NN>.md files written by the adapter → success; wrong count → failure
- LLM exit != 0 aborts
- cold-start path still succeeds
- subagent prompt ships under .agents/agents/
"""

from __future__ import annotations

import os
import re
import shutil
import subprocess
from pathlib import Path

import pytest

EVOLVE_SRC = Path(__file__).parent.parent
REPO_ROOT = EVOLVE_SRC.parent
WRITER = EVOLVE_SRC / "scripts" / "hypothesis-writer.py"
STUB = EVOLVE_SRC / "tests" / "_mocks" / "llm-stub.sh"


def _setup_evolve(tmp_path: Path, *, with_ledger: bool = False, with_leaderboard: bool = False) -> Path:
    """Copy a minimal evolve/ into tmp_path and return that EVOLVE_ROOT."""
    dst = tmp_path / "evolve"
    shutil.copytree(
        EVOLVE_SRC,
        dst,
        ignore=shutil.ignore_patterns(
            ".venv", "cycles", "ledger", "__pycache__", "*.pyc", "logs"
        ),
    )
    (dst / "ledger").mkdir(parents=True, exist_ok=True)
    if with_ledger:
        (dst / "ledger" / "cycles.jsonl").write_text(
            '{"cycle":"0001","worker_id":"local","cohort_size":2,"pareto_front":["v001"]}\n'
            '{"cycle":"0002","worker_id":"local","cohort_size":2,"pareto_front":["v001","v002"]}\n',
            encoding="utf-8",
        )
    else:
        (dst / "ledger" / "cycles.jsonl").write_text("", encoding="utf-8")
    if with_leaderboard:
        (dst / "leaderboard.md").write_text(
            "# Leaderboard\n\n| variant | hit_rate |\n|---|---|\n| v001 | 0.42 |\n",
            encoding="utf-8",
        )
    return dst


def _run_writer(
    n: int,
    evolve_root: Path,
    *,
    stub_env: dict[str, str] | None = None,
    llm_cmd: str | None = None,
    cycle_num: str = "0001",
) -> subprocess.CompletedProcess:
    env = {
        **os.environ,
        "EVOLVE_ROOT": str(evolve_root),
        "REPO_ROOT": str(REPO_ROOT),
        "EVOLVE_LLM_CMD": llm_cmd or str(STUB),
        "CYCLE_NUM": cycle_num,
    }
    if stub_env:
        env.update(stub_env)
    return subprocess.run(
        [str(WRITER), str(n)],
        capture_output=True,
        text=True,
        env=env,
    )


def _hypothesis_files(output_dir: Path) -> list[Path]:
    return sorted(p for p in output_dir.iterdir() if re.fullmatch(r"H\d{2}\.md", p.name))


def test_emits_n_distinct_files(tmp_path):
    """Happy path: exactly N H<NN>.md files in the output dir."""
    evolve = _setup_evolve(tmp_path, with_ledger=True, with_leaderboard=True)
    result = _run_writer(3, evolve, stub_env={"STUB_LINES": "3"})
    assert result.returncode == 0, result.stderr
    output_dir = evolve / "cycles" / "0001" / "hypotheses"
    files = _hypothesis_files(output_dir)
    assert [p.name for p in files] == ["H01.md", "H02.md", "H03.md"]
    bodies = {p.read_text() for p in files}
    assert len(bodies) == 3, "files must be distinct"


def test_too_few_files_fails(tmp_path):
    """N-1 files from the adapter aborts with a clear error."""
    evolve = _setup_evolve(tmp_path)
    result = _run_writer(3, evolve, stub_env={"STUB_LINES": "2"})
    assert result.returncode != 0
    assert "expected 3" in result.stderr.lower() or "got 2" in result.stderr.lower()


def test_too_many_files_fails(tmp_path):
    """N+1 files aborts (strict)."""
    evolve = _setup_evolve(tmp_path)
    result = _run_writer(3, evolve, stub_env={"STUB_LINES": "4"})
    assert result.returncode != 0
    assert "expected 3" in result.stderr.lower() or "got 4" in result.stderr.lower()


def test_no_files_fails(tmp_path):
    """Adapter writes zero files → wrapper aborts."""
    evolve = _setup_evolve(tmp_path)
    # No STUB_LINES, no STUB_OUT → stub writes nothing.
    result = _run_writer(2, evolve)
    assert result.returncode != 0


def test_llm_nonzero_exit_fails(tmp_path):
    """Adapter exiting non-zero aborts the cycle."""
    evolve = _setup_evolve(tmp_path)
    result = _run_writer(3, evolve, stub_env={"STUB_EXIT": "5", "STUB_LINES": "3"})
    assert result.returncode != 0
    assert "exit" in result.stderr.lower() or "adapter" in result.stderr.lower() or "llm" in result.stderr.lower()


def test_cold_start_with_empty_ledger_and_no_leaderboard(tmp_path):
    """Cold start: same code path; empty sections; succeeds."""
    evolve = _setup_evolve(tmp_path, with_ledger=False, with_leaderboard=False)
    result = _run_writer(2, evolve, stub_env={"STUB_LINES": "2"})
    assert result.returncode == 0, result.stderr
    files = _hypothesis_files(evolve / "cycles" / "0001" / "hypotheses")
    assert len(files) == 2


def test_adapter_invoked_with_expected_argv(tmp_path):
    """Capture the adapter argv + task.md: agent name + output-dir + --writable scope."""
    evolve = _setup_evolve(tmp_path)
    capture = tmp_path / "argv-capture.txt"
    result = _run_writer(
        2,
        evolve,
        stub_env={"STUB_LINES": "2", "STUB_CAPTURE": str(capture)},
    )
    assert result.returncode == 0, result.stderr
    body = capture.read_text()
    assert "agent: hypothesis-writer" in body
    output_dir = evolve / "cycles" / "0001" / "hypotheses"
    assert f"output-dir: {output_dir}" in body
    # The wrapper must pass --writable <output-dir> so the agent can write
    # there even under tighter tool-permissioning regimes.
    assert "--writable" in body
    assert str(output_dir) in body


def test_task_md_written_to_output_dir(tmp_path):
    """task.md exists in the output dir and mentions N + cycle."""
    evolve = _setup_evolve(tmp_path)
    capture = tmp_path / "argv-capture.txt"
    result = _run_writer(
        2,
        evolve,
        stub_env={"STUB_LINES": "2", "STUB_CAPTURE": str(capture)},
        cycle_num="0007",
    )
    assert result.returncode == 0, result.stderr
    body = capture.read_text()
    # task.md is written before invocation; stub captures it via STUB_CAPTURE.
    assert "----- task.md -----" in body
    # Should name the cycle + N so the agent can self-orient.
    assert "0007" in body
    assert re.search(r"\bN\b\s*[:=]?\s*2", body) or "N: 2" in body or " 2 " in body


def test_stdout_lists_hypothesis_file_paths(tmp_path):
    """Wrapper prints discovered file paths on stdout so evolve.py can skip re-globbing."""
    evolve = _setup_evolve(tmp_path)
    result = _run_writer(2, evolve, stub_env={"STUB_LINES": "2"})
    assert result.returncode == 0, result.stderr
    output_dir = evolve / "cycles" / "0001" / "hypotheses"
    out_paths = [line.strip() for line in result.stdout.splitlines() if line.strip()]
    assert len(out_paths) == 2
    assert sorted(out_paths) == [str(output_dir / "H01.md"), str(output_dir / "H02.md")]


def test_cold_cycle_e2e_writes_real_hypothesis_files(tmp_path):
    """E2E AC: a cold cycle with no HYPOTHESES= writes real, distinct HNN.md
    files derived from the (stub) LLM, not placeholder strings.

    Runs the orchestrator (evolve.py) end-to-end against a stubbed
    EVOLVE_LLM_CMD that writes canned hypothesis files.
    """
    repo_root_dst = tmp_path / "repo"
    shutil.copytree(
        EVOLVE_SRC,
        repo_root_dst / "evolve",
        ignore=shutil.ignore_patterns(
            ".venv", "cycles", "ledger", "__pycache__", "*.pyc"
        ),
    )

    evolve_py = EVOLVE_SRC / "scripts" / "evolve.py"
    env = {
        **os.environ,
        "EVOLVE_ROOT": str(repo_root_dst / "evolve"),
        "REPO_ROOT": str(REPO_ROOT),
        "COHORT_SIZE": "2",
        "EVOLVE_LLM_CMD": str(STUB),
        "STUB_OUT": (
            "Stash-bias eviction lifts hot-set hit rate on Zipfian traces.\n"
            "Lazy promotion cuts per-op time on read-heavy workloads."
        ),
    }
    result = subprocess.run(
        [str(evolve_py)],
        capture_output=True,
        text=True,
        env=env,
    )
    assert result.returncode == 0, result.stderr

    h01 = (repo_root_dst / "evolve" / "cycles" / "0001" / "hypotheses" / "H01.md").read_text()
    h02 = (repo_root_dst / "evolve" / "cycles" / "0001" / "hypotheses" / "H02.md").read_text()
    assert "Stash-bias eviction" in h01
    assert "Lazy promotion" in h02
    # The deleted placeholder text must not resurface.
    assert "walking-skeleton stub" not in h01
    assert "walking-skeleton stub" not in h02
    assert h01 != h02


def test_subagent_markdown_exists():
    """hypothesis-writer subagent ships under .agents/agents/ (project-scoped, agent-agnostic)."""
    agent_md = REPO_ROOT / ".agents" / "agents" / "hypothesis-writer.md"
    assert agent_md.is_file(), f"missing {agent_md}"
    body = agent_md.read_text()
    assert "hypothesis" in body.lower()
    # Must spec the file-out contract the wrapper depends on.
    assert "H01.md" in body or "H<NN>.md" in body
    assert "task.md" in body.lower()
