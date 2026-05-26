"""Tests for src/variant/repair_loop.implement_with_repair.

The repair loop drives implementer → gates up to `max_attempts` times,
returning the final GatesResult. A `_AttemptVaryingAdapter` switches its
written output per attempt number (extracted from the workspace path) so
each test exercises a different repair-loop branch without real cargo.

Cargo is mocked via tests/_mocks/cargo (same as test_gates.py).
"""

from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

import pytest

from src._io import read_utf8, write_utf8
from src.identifiers import CycleId, HypothesisId, VariantId
from src.paths import CycleVariant, Layout, SourcesLayout, VariantDir
from src.variant.repair_loop import VariantStepFailed, implement_with_repair

EVOLVE_ROOT = Path(__file__).parent.parent.parent
FIXTURES_DIR = Path(__file__).parent.parent / "fixtures"
REPO_ROOT = EVOLVE_ROOT.parent

VARIANT = CycleVariant(CycleId(1), VariantId(1))
HYPOTHESIS = HypothesisId(1)


def _setup_layout(tmp_path: Path) -> Layout:
    layout = Layout(evolve_root=tmp_path / "evolve", outputs_root=tmp_path / "out")
    layout.evolve_root.mkdir(parents=True, exist_ok=True)
    layout.sources.rust_scaffold_dir.parent.mkdir(parents=True, exist_ok=True)
    layout.sources.rust_scaffold_dir.symlink_to(SourcesLayout(root=EVOLVE_ROOT).rust_scaffold_dir)
    # Baseline parent must exist on disk so hypothesis parser can resolve it.
    baseline_impl = layout.sources.baseline.impl_dir
    baseline_impl.mkdir(parents=True, exist_ok=True)
    shutil.copy(FIXTURES_DIR / "good" / "impl" / "segment.rs", baseline_impl / "segment.rs")
    shutil.copy(FIXTURES_DIR / "good" / "impl" / "manifest.toml", baseline_impl / "manifest.toml")
    return layout


def _write_hypothesis(layout: Layout) -> Path:
    cycle_layout = layout.outputs.cycle(VARIANT.cycle_id)
    hypothesis_file = cycle_layout.hypothesis_file(HYPOTHESIS)
    hypothesis_file.parent.mkdir(parents=True, exist_ok=True)
    write_utf8(
        hypothesis_file,
        "# H01\n\ntestable claim\n\n## Parent\n\nbaseline\n\n## Changes\n\nstuff\n",
    )
    return hypothesis_file


class _AttemptVaryingAdapter:
    """Adapter that picks a fixture per attempt by inspecting the workspace
    path. Attempt N reads `outputs[N-1]` from the constructor list."""

    def __init__(self, outputs: list[str]) -> None:
        self.outputs = outputs
        self.invocation_count = 0

    def identity(self) -> str:
        return "_AttemptVaryingAdapter"

    def run(self, agent_name, workspace: Path, *, readable_roots, allow_edits) -> None:
        self.invocation_count += 1
        # workspace = .../variants/vNNN/attempts/NN/impl
        attempt_n = int(workspace.parent.name)
        outcome = self.outputs[attempt_n - 1]
        if outcome == "raise":
            raise subprocess.CalledProcessError(1, "stub-adapter")
        if outcome == "silent":
            return
        fixture = FIXTURES_DIR / outcome / "impl"
        if not fixture.is_dir():
            raise FileNotFoundError(f"unknown fixture: {outcome}")
        for name in ("segment.rs", "manifest.toml"):
            shutil.copy(fixture / name, workspace / name)


def test_initial_attempt_passes_no_repair_invoked(tmp_path):
    layout = _setup_layout(tmp_path)
    hypothesis_file = _write_hypothesis(layout)
    variant_dir = layout.outputs.cycle(VARIANT.cycle_id).variant(VARIANT.variant_id)
    adapter = _AttemptVaryingAdapter(["good"])

    result = implement_with_repair(
        hypothesis_file, variant_dir,
        variant=VARIANT, layout=layout, repo_root=REPO_ROOT,
        adapter=adapter, max_attempts=3, hypothesis=HYPOTHESIS,
    )

    assert result.all_passed
    assert adapter.invocation_count == 1
    assert (variant_dir.attempts_dir / "01").is_dir()
    assert not (variant_dir.attempts_dir / "02").exists()


def test_initial_fails_contract_gate_attempt_two_passes(tmp_path):
    layout = _setup_layout(tmp_path)
    hypothesis_file = _write_hypothesis(layout)
    variant_dir = layout.outputs.cycle(VARIANT.cycle_id).variant(VARIANT.variant_id)
    # Stage attempt 1 to have a broken eviction policy via post-hoc rewrite
    # is awkward; instead, use the "bad-invariant" fixture for the loop to
    # exercise repair, then "good" on retry. (Contract-gate failure path is
    # tested separately below.)
    adapter = _AttemptVaryingAdapter(["bad-invariant", "good"])

    result = implement_with_repair(
        hypothesis_file, variant_dir,
        variant=VARIANT, layout=layout, repo_root=REPO_ROOT,
        adapter=adapter, max_attempts=3, hypothesis=HYPOTHESIS,
    )

    assert result.all_passed
    assert adapter.invocation_count == 2
    assert (variant_dir.attempts_dir / "01").is_dir()
    assert (variant_dir.attempts_dir / "02").is_dir()
    rejection_content = read_utf8(variant_dir.rejection_md)
    assert rejection_content.count("attempt=1") == 1
    assert "attempt=2" not in rejection_content


def test_all_attempts_fail_invariant_gate_returns_failed_result(tmp_path):
    layout = _setup_layout(tmp_path)
    hypothesis_file = _write_hypothesis(layout)
    variant_dir = layout.outputs.cycle(VARIANT.cycle_id).variant(VARIANT.variant_id)
    adapter = _AttemptVaryingAdapter(["bad-invariant", "bad-invariant", "bad-invariant"])

    result = implement_with_repair(
        hypothesis_file, variant_dir,
        variant=VARIANT, layout=layout, repo_root=REPO_ROOT,
        adapter=adapter, max_attempts=3, hypothesis=HYPOTHESIS,
    )

    assert not result.all_passed
    assert result.first_failed_gate == "invariant gate"
    assert adapter.invocation_count == 3
    for n in (1, 2, 3):
        assert (variant_dir.attempts_dir / f"{n:02d}").is_dir()
    rejection_content = read_utf8(variant_dir.rejection_md)
    for n in (1, 2, 3):
        assert f"attempt={n}" in rejection_content


def test_initial_fails_build_gate_triggers_repair(tmp_path):
    layout = _setup_layout(tmp_path)
    hypothesis_file = _write_hypothesis(layout)
    variant_dir = layout.outputs.cycle(VARIANT.cycle_id).variant(VARIANT.variant_id)
    adapter = _AttemptVaryingAdapter(["broken", "good"])

    result = implement_with_repair(
        hypothesis_file, variant_dir,
        variant=VARIANT, layout=layout, repo_root=REPO_ROOT,
        adapter=adapter, max_attempts=3, hypothesis=HYPOTHESIS,
    )

    assert result.all_passed
    assert adapter.invocation_count == 2
    rejection_content = read_utf8(variant_dir.rejection_md)
    assert "build gate" in rejection_content


def test_implementer_raises_on_first_attempt_no_retry(tmp_path):
    layout = _setup_layout(tmp_path)
    hypothesis_file = _write_hypothesis(layout)
    variant_dir = layout.outputs.cycle(VARIANT.cycle_id).variant(VARIANT.variant_id)
    adapter = _AttemptVaryingAdapter(["silent", "good", "good"])

    with pytest.raises(VariantStepFailed) as excinfo:
        implement_with_repair(
            hypothesis_file, variant_dir,
            variant=VARIANT, layout=layout, repo_root=REPO_ROOT,
            adapter=adapter, max_attempts=3, hypothesis=HYPOTHESIS,
        )

    assert excinfo.value.reason == "implement"
    assert adapter.invocation_count == 1
    assert not (variant_dir.attempts_dir / "02").exists()
