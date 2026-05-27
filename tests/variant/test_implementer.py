"""Integration tests for src/evolve/variant/implementer.py::implement_variant."""

from __future__ import annotations

import shutil
from pathlib import Path

import pytest

from evolve._io import read_utf8, write_utf8
from evolve.identifiers import CycleId, HypothesisId, VariantId
from evolve.paths import Layout
from evolve.variant.implementer import implement_variant

from _stubs import StubLLMAdapter

REPO_SRC = Path(__file__).parent.parent.parent


def _setup_evolve(tmp_path: Path) -> Path:
    dst = tmp_path / "repo"
    shutil.copytree(
        REPO_SRC,
        dst,
        ignore=shutil.ignore_patterns(
            ".venv", ".git", "target", "outputs", "traces",
            "__pycache__", "*.pyc",
        ),
    )
    (dst / "outputs" / "cycles").mkdir(parents=True)
    return dst


def _write_hypothesis(path: Path, *, parent_variant: str, body: str = "test claim") -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    write_utf8(
        path,
        f"# H01\n\n{body}\n\n## Parent\n\n{parent_variant}\n\n## Changes\n\nstuff\n",
    )


def test_baseline_parent_appears_in_task_md_and_readable_roots(tmp_path):
    evolve = _setup_evolve(tmp_path)
    layout = Layout(repo_root=evolve, outputs_root=evolve / "outputs")
    cycle_layout = layout.outputs.cycle(CycleId(1))
    variant_dir = cycle_layout.variant(VariantId(1))

    hypothesis = cycle_layout.hypothesis_file(HypothesisId(1))
    _write_hypothesis(hypothesis, parent_variant="baseline")

    capture = tmp_path / "capture.txt"
    implement_variant(
        hypothesis,
        variant_dir,
        layout=layout,
        adapter=StubLLMAdapter(capture_file=capture),
    )

    body = read_utf8(capture)
    baseline_root = layout.sources.baseline.root
    assert str(baseline_root) in body, "task.md should embed baseline parent dir"
    assert str(baseline_root) in body.split("readable-roots:")[1].split("\n")[0]


def test_past_variant_parent_appears_in_task_md_and_readable_roots(tmp_path):
    evolve = _setup_evolve(tmp_path)
    layout = Layout(repo_root=evolve, outputs_root=evolve / "outputs")

    # Materialize a past variant cycle/0001/v001 so the parser can resolve it.
    past_variant_dir = layout.outputs.cycle(CycleId(1)).variant(VariantId(1)).root
    past_variant_dir.mkdir(parents=True)

    current_cycle = layout.outputs.cycle(CycleId(2))
    current_variant = current_cycle.variant(VariantId(1))
    hypothesis = current_cycle.hypothesis_file(HypothesisId(1))
    _write_hypothesis(hypothesis, parent_variant="cycle/0001/v001")

    capture = tmp_path / "capture.txt"
    implement_variant(
        hypothesis,
        current_variant,
        layout=layout,
        adapter=StubLLMAdapter(capture_file=capture),
    )

    body = read_utf8(capture)
    assert str(past_variant_dir) in body
    readable_line = body.split("readable-roots:")[1].split("\n")[0]
    assert str(past_variant_dir) in readable_line


class _SilentNoOpAdapter:
    """Adapter that returns without writing any files — regression vector for silent agents."""

    def identity(self) -> str:
        return "SilentNoOpAdapter"

    def run(self, agent_name, workspace, *, readable_roots, allow_edits) -> None:
        return None


def test_silent_adapter_raises_when_required_outputs_missing(tmp_path):
    evolve = _setup_evolve(tmp_path)
    layout = Layout(repo_root=evolve, outputs_root=evolve / "outputs")
    cycle_layout = layout.outputs.cycle(CycleId(1))
    variant_dir = cycle_layout.variant(VariantId(1))

    hypothesis = cycle_layout.hypothesis_file(HypothesisId(1))
    _write_hypothesis(hypothesis, parent_variant="baseline")

    with pytest.raises(FileNotFoundError, match="adapter did not produce"):
        implement_variant(
            hypothesis,
            variant_dir,
            layout=layout,
            adapter=_SilentNoOpAdapter(),
        )


def test_malformed_hypothesis_aborts_before_adapter(tmp_path):
    evolve = _setup_evolve(tmp_path)
    layout = Layout(repo_root=evolve, outputs_root=evolve / "outputs")
    cycle_layout = layout.outputs.cycle(CycleId(1))
    variant_dir = cycle_layout.variant(VariantId(1))

    hypothesis = cycle_layout.hypothesis_file(HypothesisId(1))
    hypothesis.parent.mkdir(parents=True, exist_ok=True)
    write_utf8(hypothesis, "just a claim line\n\n## Changes\n\nstuff\n")

    capture = tmp_path / "capture.txt"
    adapter = StubLLMAdapter(capture_file=capture)
    with pytest.raises(ValueError, match="missing '## Parent'"):
        implement_variant(hypothesis, variant_dir, layout=layout, adapter=adapter)
    assert not capture.exists(), "adapter must not be invoked when hypothesis is malformed"
