"""Integration tests for src/evolve/variant/implementer.py::implement_variant."""

from __future__ import annotations

from pathlib import Path

import pytest

from evolve._io import read_utf8, write_utf8
from evolve.config import AuthoringScope
from evolve.identifiers import CycleId, HypothesisId, VariantId
from evolve.paths import Layout
from evolve.variant.implementer import implement_variant

from _stubs import StubLLMAdapter
from _support import STUB_SUBJECT_NAME, StubSubject, write_stub_subject


def _setup_evolve(tmp_path: Path) -> tuple[Path, StubSubject]:
    # Scope/contract enforcement is driven entirely by the subject manifest, so
    # a minimal stub subject exercises the same code path as any real subject
    # without assuming a particular one.
    dst = tmp_path / "repo"
    dst.mkdir()
    (dst / "outputs" / "cycles").mkdir(parents=True)
    stub = write_stub_subject(dst / "inputs" / "subjects" / STUB_SUBJECT_NAME)
    return dst, stub


def _write_hypothesis(path: Path, *, parent_variant: str, body: str = "test claim") -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    write_utf8(
        path,
        f"# H01\n\n{body}\n\n## Parent\n\n{parent_variant}\n\n## Changes\n\nstuff\n",
    )


def test_baseline_parent_appears_in_task_md_and_readable_roots(tmp_path):
    evolve, stub = _setup_evolve(tmp_path)
    layout = Layout(repo_root=evolve, outputs_root=evolve / "outputs", subject=stub.name)
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
        authoring_scope=AuthoringScope.FULL_AUTONOMY,
    )

    body = read_utf8(capture)
    baseline_root = layout.sources.baseline.root
    assert str(baseline_root) in body, "task.md should embed baseline parent dir"
    assert str(baseline_root) in body.split("readable-roots:")[1].split("\n")[0]


def test_past_variant_parent_appears_in_task_md_and_readable_roots(tmp_path):
    evolve, stub = _setup_evolve(tmp_path)
    layout = Layout(repo_root=evolve, outputs_root=evolve / "outputs", subject=stub.name)

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
        authoring_scope=AuthoringScope.FULL_AUTONOMY,
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
    evolve, stub = _setup_evolve(tmp_path)
    layout = Layout(repo_root=evolve, outputs_root=evolve / "outputs", subject=stub.name)
    cycle_layout = layout.outputs.cycle(CycleId(1))
    variant_dir = cycle_layout.variant(VariantId(1))

    hypothesis = cycle_layout.hypothesis_file(HypothesisId(1))
    _write_hypothesis(hypothesis, parent_variant="baseline")

    with pytest.raises(FileNotFoundError, match="adapter produced no overlay files"):
        implement_variant(
            hypothesis,
            variant_dir,
            layout=layout,
            adapter=_SilentNoOpAdapter(),
            authoring_scope=AuthoringScope.EXTENSION_POINTS,
        )


def test_malformed_hypothesis_aborts_before_adapter(tmp_path):
    evolve, stub = _setup_evolve(tmp_path)
    layout = Layout(repo_root=evolve, outputs_root=evolve / "outputs", subject=stub.name)
    cycle_layout = layout.outputs.cycle(CycleId(1))
    variant_dir = cycle_layout.variant(VariantId(1))

    hypothesis = cycle_layout.hypothesis_file(HypothesisId(1))
    hypothesis.parent.mkdir(parents=True, exist_ok=True)
    write_utf8(hypothesis, "just a claim line\n\n## Changes\n\nstuff\n")

    capture = tmp_path / "capture.txt"
    adapter = StubLLMAdapter(capture_file=capture)
    with pytest.raises(ValueError, match="missing '## Parent'"):
        implement_variant(
            hypothesis, variant_dir, layout=layout, adapter=adapter,
            authoring_scope=AuthoringScope.FULL_AUTONOMY,
        )
    assert not capture.exists(), "adapter must not be invoked when hypothesis is malformed"


class _WritingAdapter:
    """Adapter that writes a fixed set of overlay files (relative to overlay/)."""

    def __init__(self, rel_paths: list[str]) -> None:
        self.rel_paths = rel_paths

    def identity(self) -> str:
        return "WritingAdapter"

    def run(self, agent_name, workspace, *, readable_roots, allow_edits) -> None:
        for rel in self.rel_paths:
            target = workspace / "overlay" / rel
            target.parent.mkdir(parents=True, exist_ok=True)
            write_utf8(target, "// stub\n")


class _EmptySubdirAdapter:
    """Adapter that creates only an empty subdirectory in the overlay — no files."""

    def identity(self) -> str:
        return "EmptySubdirAdapter"

    def run(self, agent_name, workspace, *, readable_roots, allow_edits) -> None:
        (workspace / "overlay" / "src").mkdir(parents=True, exist_ok=True)


def _prepare(tmp_path: Path):
    evolve, stub = _setup_evolve(tmp_path)
    layout = Layout(repo_root=evolve, outputs_root=evolve / "outputs", subject=stub.name)
    cycle_layout = layout.outputs.cycle(CycleId(1))
    variant_dir = cycle_layout.variant(VariantId(1))
    hypothesis = cycle_layout.hypothesis_file(HypothesisId(1))
    _write_hypothesis(hypothesis, parent_variant="baseline")
    return layout, variant_dir, hypothesis, stub


def test_extension_points_scope_allows_whitelisted_file(tmp_path):
    layout, variant_dir, hypothesis, stub = _prepare(tmp_path)
    implement_variant(
        hypothesis, variant_dir, layout=layout,
        adapter=_WritingAdapter([stub.extension_point_file.as_posix()]),
        authoring_scope=AuthoringScope.EXTENSION_POINTS,
    )
    assert (variant_dir.overlay_dir / stub.extension_point_file).is_file()


def test_extension_points_scope_rejects_non_whitelisted_file(tmp_path):
    layout, variant_dir, hypothesis, stub = _prepare(tmp_path)
    with pytest.raises(ValueError, match="extension-points"):
        implement_variant(
            hypothesis, variant_dir, layout=layout,
            adapter=_WritingAdapter([stub.free_file.as_posix()]),
            authoring_scope=AuthoringScope.EXTENSION_POINTS,
        )


def test_full_autonomy_scope_allows_any_non_protected_file(tmp_path):
    layout, variant_dir, hypothesis, stub = _prepare(tmp_path)
    implement_variant(
        hypothesis, variant_dir, layout=layout,
        adapter=_WritingAdapter([stub.free_file.as_posix()]),
        authoring_scope=AuthoringScope.FULL_AUTONOMY,
    )
    assert (variant_dir.overlay_dir / stub.free_file).is_file()


def test_full_autonomy_scope_rejects_protected_file(tmp_path):
    layout, variant_dir, hypothesis, stub = _prepare(tmp_path)
    with pytest.raises(ValueError, match="contract-protected"):
        implement_variant(
            hypothesis, variant_dir, layout=layout,
            adapter=_WritingAdapter([stub.protected_files[1].as_posix()]),
            authoring_scope=AuthoringScope.FULL_AUTONOMY,
        )


def test_extension_points_scope_rejects_protected_file(tmp_path):
    layout, variant_dir, hypothesis, stub = _prepare(tmp_path)
    with pytest.raises(ValueError, match="contract-protected"):
        implement_variant(
            hypothesis, variant_dir, layout=layout,
            adapter=_WritingAdapter([stub.protected_files[0].as_posix()]),
            authoring_scope=AuthoringScope.EXTENSION_POINTS,
        )


def test_empty_subdir_overlay_raises(tmp_path):
    layout, variant_dir, hypothesis, stub = _prepare(tmp_path)
    with pytest.raises(FileNotFoundError, match="adapter produced no overlay files"):
        implement_variant(
            hypothesis, variant_dir, layout=layout,
            adapter=_EmptySubdirAdapter(),
            authoring_scope=AuthoringScope.FULL_AUTONOMY,
        )


def test_task_md_contains_manifest_path_and_scope(tmp_path):
    layout, variant_dir, hypothesis, stub = _prepare(tmp_path)
    implement_variant(
        hypothesis, variant_dir, layout=layout,
        adapter=_WritingAdapter([stub.extension_point_file.as_posix()]),
        authoring_scope=AuthoringScope.EXTENSION_POINTS,
    )
    body = read_utf8(variant_dir.task_md)
    assert str(layout.sources.subject_manifest) in body
    assert "extension-points" in body
