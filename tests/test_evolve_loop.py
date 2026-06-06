"""Tests for the N-cycle loop in evolve."""

import shutil
from pathlib import Path

import pytest

from evolve.cli import run_cycle
from evolve._io import read_utf8
from evolve.config import load_configs
from evolve.identifiers import CycleId
from evolve.paths import Layout

from _stubs import StubLLMAdapter
from _support import configured_subject

REPO_SRC = Path(__file__).parent.parent
SUBJECT = configured_subject(REPO_SRC)
COHORT = 2


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


def _run_cycles(repo_root: Path, n: int) -> None:
    # Stub overlay writes a `.overlay` sentinel, not a manifest impl file;
    # full-autonomy keeps it legal so the loop mechanics aren't scope-gated.
    # The loop runs on a StubLLMAdapter, so the backend is irrelevant here; pin
    # ollama-local (the lightest, pure-Python backend) so config loads.
    cfg = load_configs(
        repo_root,
        cohort_size=COHORT,
        authoring_scope="full-autonomy",
        ai_agent_tool="ollama-local",
        model="gemma4:e4b",
    )
    for cycle_index in range(1, n + 1):
        # From cycle 2 on, the prior cycle's variants sit on the global front, so
        # the parent-anchor quota requires at least one hypothesis to parent on a
        # non-baseline incumbent; cite the previous cycle's v001.
        parent_variants = [f"cycle/{cycle_index - 1:04d}/v001"] if cycle_index > 1 else None
        run_cycle(
            layout=_layout(repo_root),
            repo_root=repo_root,
            adapter=StubLLMAdapter(n_lines=COHORT, parent_variants=parent_variants),
            cfg=cfg,
        )


@pytest.mark.skip(reason="cycle seal stage is disabled in run_cycle (cli.py); re-enable seal_cycle to restore")
@pytest.mark.slow
def test_two_cycles_both_sealed(tmp_path):
    repo_root = _setup_repo(tmp_path)
    _run_cycles(repo_root, 2)

    layout = _layout(repo_root)
    for n in ("0001", "0002"):
        cycle_layout = layout.outputs.cycle(n)
        assert cycle_layout.root.is_dir(), f"missing cycle {n}"
        assert cycle_layout.sealed_marker.is_file(), f"cycle {n} not sealed"

    rows = read_utf8(layout.outputs.ledger).splitlines()
    assert len(rows) == 2


@pytest.mark.slow
def test_single_cycle_default(tmp_path):
    repo_root = _setup_repo(tmp_path)
    _run_cycles(repo_root, 1)
    cycles_root = _layout(repo_root).outputs.cycles_dir
    cycles = sorted(p.name for p in cycles_root.iterdir() if p.is_dir())
    assert cycles == ["0001"]


def test_loop_aborts_on_cycle_failure(tmp_path, monkeypatch):
    repo_root = _setup_repo(tmp_path)

    def failing(*_args, **_kwargs):
        raise RuntimeError("boom")

    monkeypatch.setattr("evolve.variant.implementer.implement_variant", failing)
    with pytest.raises(SystemExit):
        _run_cycles(repo_root, 2)

    assert not _layout(repo_root).outputs.cycle(CycleId(2)).root.exists()
