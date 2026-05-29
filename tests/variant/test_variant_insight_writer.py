"""Tests for src/evolve/variant/variant_insight_writer.py::write_insight."""

from __future__ import annotations

from pathlib import Path

import pytest

from evolve._io import read_utf8, write_utf8
from evolve.identifiers import CycleId, HypothesisId, VariantId
from evolve.paths import Layout
from evolve.variant.variant_insight_writer import AGENT_NAME, write_insight

from _support import STUB_SUBJECT_NAME

REPO_SRC = Path(__file__).parent.parent.parent


def _setup_evolve(tmp_path: Path) -> Path:
    # write_insight only emits subject-scoped baseline paths into task.md (it
    # never reads the subject tree), so a bare outputs dir plus a stub subject
    # name is enough.
    dst = tmp_path / "repo"
    (dst / "outputs" / "cycles").mkdir(parents=True)
    return dst


def _write_hypothesis(path: Path, *, parent_variant: str = "baseline") -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    write_utf8(
        path,
        f"# H01\n\ntest claim\n\n## Parent\n\n{parent_variant}\n\n## Changes\n\nstuff\n",
    )


class _RecordingAdapter:
    """Adapter that records the call and writes the required insight.md."""

    def __init__(self) -> None:
        self.calls: list[dict] = []

    def identity(self) -> str:
        return "RecordingAdapter"

    def run(self, agent_name, workspace, *, readable_roots, allow_edits) -> None:
        self.calls.append({
            "agent_name": agent_name,
            "workspace": workspace,
            "readable_roots": list(readable_roots),
            "allow_edits": allow_edits,
            "task_md": read_utf8(workspace / "task.md"),
        })
        write_utf8(workspace / "insight.md", "# Variant Insight\n\nstub\n")


class _SilentAdapter:
    def identity(self) -> str:
        return "SilentAdapter"

    def run(self, agent_name, workspace, *, readable_roots, allow_edits) -> None:
        return None


def test_task_md_lists_current_parent_and_hypothesis_paths(tmp_path):
    evolve = _setup_evolve(tmp_path)
    layout = Layout(repo_root=evolve, outputs_root=evolve / "outputs", subject=STUB_SUBJECT_NAME)
    cycle_layout = layout.outputs.cycle(CycleId(1))
    variant_dir = cycle_layout.variant(VariantId(1))
    variant_dir.root.mkdir(parents=True)

    hypothesis = cycle_layout.hypothesis_file(HypothesisId(1))
    _write_hypothesis(hypothesis, parent_variant="baseline")

    adapter = _RecordingAdapter()
    write_insight(variant_dir, hypothesis, layout=layout, adapter=adapter)

    task_md = adapter.calls[0]["task_md"]
    baseline_dir = layout.sources.baseline
    assert str(variant_dir.root) in task_md
    assert str(variant_dir.overlay_dir) in task_md
    assert str(variant_dir.fitness_vector_json) in task_md
    assert str(variant_dir.eval_dir) in task_md
    assert str(baseline_dir.root) in task_md
    assert str(baseline_dir.overlay_dir) in task_md
    assert str(baseline_dir.fitness_vector_json) in task_md
    assert str(baseline_dir.insight_md) in task_md
    assert str(hypothesis) in task_md


def test_readable_roots_matches_spec(tmp_path):
    evolve = _setup_evolve(tmp_path)
    layout = Layout(repo_root=evolve, outputs_root=evolve / "outputs", subject=STUB_SUBJECT_NAME)
    cycle_layout = layout.outputs.cycle(CycleId(1))
    variant_dir = cycle_layout.variant(VariantId(1))
    variant_dir.root.mkdir(parents=True)

    hypothesis = cycle_layout.hypothesis_file(HypothesisId(1))
    _write_hypothesis(hypothesis, parent_variant="baseline")

    adapter = _RecordingAdapter()
    write_insight(variant_dir, hypothesis, layout=layout, adapter=adapter)

    baseline_dir = layout.sources.baseline
    assert adapter.calls[0]["readable_roots"] == [
        variant_dir.root,
        baseline_dir.root,
        hypothesis,
    ]
    assert adapter.calls[0]["agent_name"] == AGENT_NAME
    assert adapter.calls[0]["allow_edits"] is True


def test_missing_insight_md_raises(tmp_path):
    evolve = _setup_evolve(tmp_path)
    layout = Layout(repo_root=evolve, outputs_root=evolve / "outputs", subject=STUB_SUBJECT_NAME)
    cycle_layout = layout.outputs.cycle(CycleId(1))
    variant_dir = cycle_layout.variant(VariantId(1))
    variant_dir.root.mkdir(parents=True)

    hypothesis = cycle_layout.hypothesis_file(HypothesisId(1))
    _write_hypothesis(hypothesis, parent_variant="baseline")

    with pytest.raises(FileNotFoundError, match="insight.md"):
        write_insight(variant_dir, hypothesis, layout=layout, adapter=_SilentAdapter())


def test_past_cycle_variant_parent_resolves(tmp_path):
    evolve = _setup_evolve(tmp_path)
    layout = Layout(repo_root=evolve, outputs_root=evolve / "outputs", subject=STUB_SUBJECT_NAME)

    past_variant_dir = layout.outputs.cycle(CycleId(1)).variant(VariantId(1))
    past_variant_dir.root.mkdir(parents=True)

    current_cycle = layout.outputs.cycle(CycleId(2))
    current_variant = current_cycle.variant(VariantId(1))
    current_variant.root.mkdir(parents=True)
    hypothesis = current_cycle.hypothesis_file(HypothesisId(1))
    _write_hypothesis(hypothesis, parent_variant="cycle/0001/v001")

    adapter = _RecordingAdapter()
    write_insight(current_variant, hypothesis, layout=layout, adapter=adapter)

    task_md = adapter.calls[0]["task_md"]
    assert str(past_variant_dir.root) in task_md
    assert adapter.calls[0]["readable_roots"] == [
        current_variant.root,
        past_variant_dir.root,
        hypothesis,
    ]


def test_success_path_returns_none(tmp_path):
    evolve = _setup_evolve(tmp_path)
    layout = Layout(repo_root=evolve, outputs_root=evolve / "outputs", subject=STUB_SUBJECT_NAME)
    cycle_layout = layout.outputs.cycle(CycleId(1))
    variant_dir = cycle_layout.variant(VariantId(1))
    variant_dir.root.mkdir(parents=True)

    hypothesis = cycle_layout.hypothesis_file(HypothesisId(1))
    _write_hypothesis(hypothesis, parent_variant="baseline")

    result = write_insight(variant_dir, hypothesis, layout=layout, adapter=_RecordingAdapter())
    assert result is None
    assert variant_dir.insight_md.is_file()
