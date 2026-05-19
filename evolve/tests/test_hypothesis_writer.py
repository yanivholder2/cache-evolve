"""Tests for evolve/scripts/hypothesis-writer.py.

Covers the contract:
- emits exactly N non-empty lines on stdout
- fails loudly on wrong line count, empty output, or LLM exit != 0
- assembles a prompt containing leaderboard / ledger / extension-points sections
- cold-start path: empty ledger + leaderboard still produces a valid prompt
"""

from __future__ import annotations

import os
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
) -> subprocess.CompletedProcess:
    env = {
        **os.environ,
        "EVOLVE_ROOT": str(evolve_root),
        "REPO_ROOT": str(REPO_ROOT),
        "EVOLVE_LLM_CMD": llm_cmd or str(STUB),
    }
    if stub_env:
        env.update(stub_env)
    return subprocess.run(
        [str(WRITER), str(n)],
        capture_output=True,
        text=True,
        env=env,
    )


def test_emits_n_distinct_lines(tmp_path):
    """Happy path: exactly N non-empty lines on stdout."""
    evolve = _setup_evolve(tmp_path, with_ledger=True, with_leaderboard=True)
    result = _run_writer(3, evolve, stub_env={"STUB_LINES": "3"})
    assert result.returncode == 0, result.stderr
    lines = result.stdout.splitlines()
    assert len(lines) == 3
    assert all(line.strip() for line in lines)


def test_too_few_lines_fails(tmp_path):
    """N-1 lines from the LLM aborts with a clear error."""
    evolve = _setup_evolve(tmp_path)
    result = _run_writer(3, evolve, stub_env={"STUB_LINES": "2"})
    assert result.returncode != 0
    assert "expected 3" in result.stderr.lower() or "got 2" in result.stderr.lower()


def test_too_many_lines_fails(tmp_path):
    """N+1 lines aborts (strict, no truncation)."""
    evolve = _setup_evolve(tmp_path)
    result = _run_writer(3, evolve, stub_env={"STUB_LINES": "4"})
    assert result.returncode != 0
    assert "expected 3" in result.stderr.lower() or "got 4" in result.stderr.lower()


def test_empty_output_fails(tmp_path):
    """Empty LLM output aborts."""
    evolve = _setup_evolve(tmp_path)
    result = _run_writer(2, evolve, stub_env={"STUB_OUT": ""})
    assert result.returncode != 0


def test_llm_nonzero_exit_fails(tmp_path):
    """LLM exiting non-zero aborts the cycle."""
    evolve = _setup_evolve(tmp_path)
    result = _run_writer(3, evolve, stub_env={"STUB_EXIT": "5", "STUB_LINES": "3"})
    assert result.returncode != 0
    assert "llm" in result.stderr.lower() or "exit" in result.stderr.lower()


def test_cold_start_with_empty_ledger_and_no_leaderboard(tmp_path):
    """Cold start: same code path; empty sections; succeeds."""
    evolve = _setup_evolve(tmp_path, with_ledger=False, with_leaderboard=False)
    result = _run_writer(2, evolve, stub_env={"STUB_LINES": "2"})
    assert result.returncode == 0, result.stderr
    assert len(result.stdout.splitlines()) == 2


def test_prompt_contains_required_sections(tmp_path):
    """Prompt assembled by the wrapper must include named context sections."""
    evolve = _setup_evolve(tmp_path, with_ledger=True, with_leaderboard=True)
    capture = tmp_path / "prompt-capture.txt"
    result = _run_writer(
        2,
        evolve,
        stub_env={"STUB_LINES": "2", "STUB_CAPTURE": str(capture)},
    )
    assert result.returncode == 0, result.stderr
    prompt = capture.read_text(encoding="utf-8")
    # Subagent body + context headers must be inlined.
    assert "hypothesis-writer" in prompt.lower()
    assert "leaderboard" in prompt.lower()
    assert "ledger" in prompt.lower()
    assert "extension-points" in prompt.lower() or "extension points" in prompt.lower()
    assert "Task" in prompt
    # Real content from the fixtures must surface, not just header names.
    assert "Segment" in prompt  # from extension-points.md
    assert "v001" in prompt or "hit_rate" in prompt  # from leaderboard.md


def test_raw_log_written_per_invocation(tmp_path):
    """Wrapper writes raw stdout to evolve/logs/cycle-NNNN-hypothesis-writer.log."""
    evolve = _setup_evolve(tmp_path)
    env_extra = {"CYCLE_NUM": "0007"}
    env = {
        **os.environ,
        "EVOLVE_ROOT": str(evolve),
        "REPO_ROOT": str(REPO_ROOT),
        "EVOLVE_LLM_CMD": str(STUB),
        "STUB_LINES": "2",
        **env_extra,
    }
    result = subprocess.run(
        [str(WRITER), "2"], capture_output=True, text=True, env=env
    )
    assert result.returncode == 0, result.stderr
    log = evolve / "logs" / "cycle-0007-hypothesis-writer.log"
    assert log.is_file(), f"missing raw log {log}"
    assert "stub hypothesis 1" in log.read_text(encoding="utf-8")


def test_cold_cycle_e2e_writes_real_hypothesis_files(tmp_path):
    """E2E AC: a cold cycle with no HYPOTHESES= writes real, distinct HNN.md
    files derived from the (stub) LLM, not placeholder strings.

    Runs the orchestrator end-to-end against a stubbed EVOLVE_LLM_CMD.
    """
    repo_root_dst = tmp_path / "repo"
    shutil.copytree(
        EVOLVE_SRC,
        repo_root_dst / "evolve",
        ignore=shutil.ignore_patterns(
            ".venv", "cycles", "ledger", "__pycache__", "*.pyc"
        ),
    )

    cycle_start = EVOLVE_SRC / "scripts" / "cycle-start.py"
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
        [str(cycle_start)],
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
    """hypothesis-writer subagent ships as plain markdown under .claude/agents/."""
    agent_md = REPO_ROOT / ".claude" / "agents" / "hypothesis-writer.md"
    assert agent_md.is_file(), f"missing {agent_md}"
    body = agent_md.read_text()
    # Spec the contract the wrapper depends on.
    assert "hypothesis" in body.lower()
    assert "one per line" in body.lower() or "one line per hypothesis" in body.lower() or "N lines" in body
