"""Integration tests for evolve/src/variant_implementer.implement_variant.

Drives the public API with the StubLLMAdapter to assert:
- the resolved parent dir appears in task.md and readable_roots
- a malformed hypothesis aborts before any adapter invocation
"""

from __future__ import annotations

import shutil
from pathlib import Path

import pytest

from src.paths import Layout
from src.variant_implementer import implement_variant

from _stubs import StubLLMAdapter

EVOLVE_SRC = Path(__file__).parent.parent


def _setup_evolve(tmp_path: Path) -> Path:
    dst = tmp_path / "evolve"
    shutil.copytree(
        EVOLVE_SRC,
        dst,
        ignore=shutil.ignore_patterns(".venv", "outputs", "__pycache__", "*.pyc"),
    )
    (dst / "outputs" / "cycles").mkdir(parents=True)
    return dst


def _write_hypothesis(path: Path, *, parent: str, body: str = "test claim") -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        f"# H01\n\n{body}\n\n## Parent\n\n{parent}\n\n## Changes\n\nstuff\n",
        encoding="utf-8",
    )


def test_baseline_parent_appears_in_task_md_and_readable_roots(tmp_path):
    evolve = _setup_evolve(tmp_path)
    layout = Layout(evolve_root=evolve, outputs_root=evolve / "outputs")
    cycle_layout = layout.cycle("0001")
    variant_layout = cycle_layout.variant(1)

    hypothesis = cycle_layout.hypothesis_file(1)
    _write_hypothesis(hypothesis, parent="baseline")

    capture = tmp_path / "capture.txt"
    implement_variant(
        hypothesis,
        variant_layout,
        layout=layout,
        adapter=StubLLMAdapter(capture_file=capture),
    )

    body = capture.read_text()
    baseline_root = layout.baseline.root
    assert str(baseline_root) in body, "task.md should embed baseline parent dir"
    assert str(baseline_root) in body.split("readable-roots:")[1].split("\n")[0]


def test_past_variant_parent_appears_in_task_md_and_readable_roots(tmp_path):
    evolve = _setup_evolve(tmp_path)
    layout = Layout(evolve_root=evolve, outputs_root=evolve / "outputs")

    # Materialize a past variant cycle/0001/v001 so resolve_parent_dir finds it.
    past_variant_dir = layout.cycle("0001").variant(1).root
    past_variant_dir.mkdir(parents=True)

    current_cycle = layout.cycle("0002")
    current_variant = current_cycle.variant(1)
    hypothesis = current_cycle.hypothesis_file(1)
    _write_hypothesis(hypothesis, parent="cycle/0001/v001")

    capture = tmp_path / "capture.txt"
    implement_variant(
        hypothesis,
        current_variant,
        layout=layout,
        adapter=StubLLMAdapter(capture_file=capture),
    )

    body = capture.read_text()
    assert str(past_variant_dir) in body
    readable_line = body.split("readable-roots:")[1].split("\n")[0]
    assert str(past_variant_dir) in readable_line


def test_malformed_hypothesis_aborts_before_adapter(tmp_path):
    evolve = _setup_evolve(tmp_path)
    layout = Layout(evolve_root=evolve, outputs_root=evolve / "outputs")
    cycle_layout = layout.cycle("0001")
    variant_layout = cycle_layout.variant(1)

    hypothesis = cycle_layout.hypothesis_file(1)
    hypothesis.parent.mkdir(parents=True, exist_ok=True)
    hypothesis.write_text("just a claim line\n\n## Changes\n\nstuff\n", encoding="utf-8")

    capture = tmp_path / "capture.txt"
    adapter = StubLLMAdapter(capture_file=capture)
    with pytest.raises(ValueError, match="missing '## Parent'"):
        implement_variant(hypothesis, variant_layout, layout=layout, adapter=adapter)
    assert not capture.exists(), "adapter must not be invoked when hypothesis is malformed"
