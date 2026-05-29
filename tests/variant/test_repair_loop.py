"""Integration tests for the self-repair sub-loop (repair_loop.implement_with_repair).

A scripted adapter writes a different overlay per attempt; cargo is mocked via
tests/_mocks/cargo, which scans the overlay for `// MOCK_CARGO:` markers to
decide each gate's pass/fail. So the loop runs the *real* implementer + gates
end-to-end without paying for Rust compilation, and we assert on the on-disk
attempts/ dirs and the cumulative rejection.md the runner writes.
"""

from __future__ import annotations

from pathlib import Path

import pytest

from evolve._io import read_utf8, write_utf8
from evolve.config import AuthoringScope
from evolve.identifiers import CycleId, HypothesisId, VariantId
from evolve.paths import CycleVariant, Layout
from evolve.variant.repair_loop import VariantStepFailed, implement_with_repair

from _support import configured_subject

REPO_ROOT = Path(__file__).parent.parent.parent
SUBJECT = configured_subject(REPO_ROOT)

# Overlay landing spot: a real, non-protected dash source file, so the
# implementer's full-autonomy scope check passes. mock cargo scans every
# `.rs` under the workspace src/ for markers, so the file's identity is
# irrelevant — only the marker content matters.
OVERLAY_REL = "src/shared/eviction_policy.rs"

GOOD = "// clean overlay, no marker\n"
BAD_INVARIANT = "// MOCK_CARGO: invariant_gate_fail P1 no_phantom_items\n"
BUILD_FAIL = "// MOCK_CARGO: build_gate_fail\n"


class _ScriptedAdapter:
    """Writes one overlay per attempt from `bodies` (indexed by call count).

    A `None` entry raises mid-run to simulate an implementer crash. Tracks
    `calls` so a test can assert the loop stopped retrying."""

    def __init__(self, bodies: list[str | None]) -> None:
        self.bodies = bodies
        self.calls = 0

    def identity(self) -> str:
        return "ScriptedAdapter"

    def run(self, agent_name, workspace, *, readable_roots, allow_edits) -> None:
        body = self.bodies[self.calls]
        self.calls += 1
        if body is None:
            raise RuntimeError("adapter crashed")
        target = workspace / "overlay" / OVERLAY_REL
        target.parent.mkdir(parents=True, exist_ok=True)
        write_utf8(target, body)


def _setup(tmp_path: Path) -> tuple[Layout, CycleVariant, Path]:
    """Layout-shaped scratch dir: a symlink farm pointing at the real subject +
    framework crate (so load_subject + the build harness resolve), plus a
    hypothesis file parented on the baseline."""

    layout = Layout(repo_root=tmp_path / "repo", outputs_root=tmp_path / "out", subject=SUBJECT)
    layout.repo_root.mkdir(parents=True, exist_ok=True)
    layout.sources.subject_dir.parent.mkdir(parents=True, exist_ok=True)
    layout.sources.subject_dir.symlink_to(REPO_ROOT / "inputs" / "subjects" / SUBJECT)
    layout.sources.framework_crate_dir.symlink_to(REPO_ROOT / "framework-rs")

    cycle_layout = layout.outputs.cycle(CycleId(1))
    hypothesis_file = cycle_layout.hypothesis_file(HypothesisId(1))
    hypothesis_file.parent.mkdir(parents=True, exist_ok=True)
    write_utf8(
        hypothesis_file,
        "# H01\n\ntest claim\n\n## Parent\n\nbaseline\n\n## Changes\n\nstuff\n",
    )
    return layout, CycleVariant(CycleId(1), VariantId(1)), hypothesis_file


def _run(layout, variant, hypothesis_file, adapter, *, attempts):
    return implement_with_repair(
        variant,
        hypothesis_file,
        layout=layout,
        repo_root=REPO_ROOT,
        adapter=adapter,
        authoring_scope=AuthoringScope.FULL_AUTONOMY,
        hypothesis_id=HypothesisId(1),
        attempts=attempts,
    )


def _attempt_count(rejection_md: Path) -> int:
    if not rejection_md.is_file():
        return 0
    return read_utf8(rejection_md).count("attempt=")


def test_initial_pass_creates_only_attempt_01(tmp_path):
    layout, variant, hypothesis_file = _setup(tmp_path)
    adapter = _ScriptedAdapter([GOOD])

    result = _run(layout, variant, hypothesis_file, adapter, attempts=3)

    assert result.all_passed
    assert adapter.calls == 1
    variant_dir = variant.resolve(layout)
    assert variant_dir.attempt(1).overlay_dir.is_dir()
    assert not variant_dir.attempt(2).root.exists()
    assert _attempt_count(variant_dir.rejection_md) == 0


def test_repair_then_pass(tmp_path):
    layout, variant, hypothesis_file = _setup(tmp_path)
    adapter = _ScriptedAdapter([BAD_INVARIANT, GOOD])

    result = _run(layout, variant, hypothesis_file, adapter, attempts=3)

    assert result.all_passed
    assert adapter.calls == 2
    variant_dir = variant.resolve(layout)
    assert variant_dir.attempt(1).overlay_dir.is_dir()
    assert variant_dir.attempt(2).overlay_dir.is_dir()
    assert not variant_dir.attempt(3).root.exists()
    # Exactly the one failed attempt is logged, tagged attempt=1.
    content = read_utf8(variant_dir.rejection_md)
    assert content.count("attempt=") == 1
    assert "attempt=1" in content


def test_exhaust_all_attempts(tmp_path):
    layout, variant, hypothesis_file = _setup(tmp_path)
    adapter = _ScriptedAdapter([BAD_INVARIANT, BAD_INVARIANT, BAD_INVARIANT])

    result = _run(layout, variant, hypothesis_file, adapter, attempts=3)

    assert not result.all_passed
    assert result.first_failed_gate == "invariant gate"
    assert adapter.calls == 3
    variant_dir = variant.resolve(layout)
    for n in (1, 2, 3):
        assert variant_dir.attempt(n).overlay_dir.is_dir()
    content = read_utf8(variant_dir.rejection_md)
    assert content.count("attempt=") == 3
    for n in (1, 2, 3):
        assert f"attempt={n}" in content


def test_build_gate_failure_triggers_repair(tmp_path):
    layout, variant, hypothesis_file = _setup(tmp_path)
    adapter = _ScriptedAdapter([BUILD_FAIL, GOOD])

    result = _run(layout, variant, hypothesis_file, adapter, attempts=3)

    assert result.all_passed
    assert adapter.calls == 2
    content = read_utf8(variant.resolve(layout).rejection_md)
    assert content.count("attempt=") == 1
    assert "build gate" in content


def test_build_env_error_hard_fails_without_burning_attempts(tmp_path, monkeypatch):
    # A non-repairable harness error (e.g. missing subject/framework crate) is
    # not a gate the agent can fix; it must propagate after one attempt rather
    # than re-invoking the implementer for the whole budget.
    layout, variant, hypothesis_file = _setup(tmp_path)
    adapter = _ScriptedAdapter([GOOD, GOOD, GOOD])

    def _raise_env_error(*args, **kwargs):
        raise FileNotFoundError("missing framework crate")

    monkeypatch.setattr(
        "evolve.variant.gates.build_gate.compile_for_gates", _raise_env_error
    )

    with pytest.raises(FileNotFoundError, match="missing framework crate"):
        _run(layout, variant, hypothesis_file, adapter, attempts=3)

    assert adapter.calls == 1


def test_implementer_crash_hard_fails_without_retry(tmp_path):
    layout, variant, hypothesis_file = _setup(tmp_path)
    adapter = _ScriptedAdapter([None, GOOD])

    with pytest.raises(VariantStepFailed) as exc_info:
        _run(layout, variant, hypothesis_file, adapter, attempts=3)

    assert exc_info.value.reason == "implement"
    # A crash is not a gate failure: the loop must not retry.
    assert adapter.calls == 1
