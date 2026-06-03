"""Tests for src/evolve/hypothesis/writer.py::write_hypotheses."""

from __future__ import annotations

import json
import re
import shutil
import subprocess
from pathlib import Path

import pytest

from evolve.cli import run_cycle
from evolve._io import read_utf8, write_utf8
from evolve.config import AuthoringScope, load_configs
from evolve.hypothesis.writer import write_hypotheses
from evolve.identifiers import CycleId, HypothesisId
from evolve.paths import Layout, SourcesLayout

from _stubs import StubLLMAdapter
from _support import configured_subject

REPO_SRC = Path(__file__).parent.parent.parent
# `_copy_real_evolve` runs a real cycle against the configured subject; the
# non-slow tests only need a stable subject name for the task.md paths they
# emit (write_hypotheses never reads the subject tree), so the same value
# serves both.
SUBJECT = configured_subject(REPO_SRC)


def _layout(evolve: Path) -> Layout:
    return Layout(repo_root=evolve, outputs_root=evolve / "outputs", subject=SUBJECT)


def _setup_evolve(tmp_path: Path, *, with_ledger: bool = False, with_leaderboard: bool = False) -> Path:
    # write_hypotheses doesn't read the evolve source tree — only references
    # baseline paths in the task.md it writes. Create the outputs the test
    # needs; skip the rest.
    dst = tmp_path / "evolve"
    dst.mkdir()
    layout = _layout(dst)
    layout.outputs.ledger.parent.mkdir(parents=True, exist_ok=True)
    if with_ledger:
        write_utf8(
            layout.outputs.ledger,
            '{"cycle":"0001","cohort_size":2,"winners":["v001"]}\n'
            '{"cycle":"0002","cohort_size":2,"winners":["v001","v002"]}\n',
        )
    else:
        write_utf8(layout.outputs.ledger, "")
    if with_leaderboard:
        write_utf8(
            layout.outputs.leaderboard_md,
            "# Leaderboard\n\n| variant | hit_rate |\n|---|---|\n| v001 | 0.42 |\n",
        )
    return dst


def _copy_real_evolve(tmp_path: Path) -> Path:
    # Full copy is only needed for the e2e slow test that drives run_cycle
    # past task.md into baseline-reading code.
    dst = tmp_path / "repo"
    shutil.copytree(
        REPO_SRC,
        dst,
        ignore=shutil.ignore_patterns(
            ".venv", ".git", "target", "outputs", "traces",
            "__pycache__", "*.pyc",
        ),
    )
    layout = _layout(dst)
    layout.outputs.ledger.parent.mkdir(parents=True, exist_ok=True)
    write_utf8(layout.outputs.ledger, "")
    return dst


def _hypothesis_files(output_dir: Path) -> list[Path]:
    return sorted(p for p in output_dir.iterdir() if re.fullmatch(r"H\d{2}\.md", p.name))


def _stage_prev_cycle(evolve: Path, prev_cycle_id: CycleId) -> None:
    # Continuation cycles require the prev cycle's variants/ dir to exist.
    _layout(evolve).outputs.cycle(prev_cycle_id).variants_dir.mkdir(parents=True, exist_ok=True)


def test_emits_n_distinct_files(tmp_path):
    evolve = _setup_evolve(tmp_path, with_ledger=True, with_leaderboard=True)
    files = write_hypotheses(3, layout=_layout(evolve), cycle_id=CycleId(1), adapter=StubLLMAdapter(n_lines=3))
    assert [p.name for p in files] == ["H01.md", "H02.md", "H03.md"]
    bodies = {read_utf8(p) for p in files}
    assert len(bodies) == 3, "files must be distinct"


def test_too_few_files_fails(tmp_path):
    evolve = _setup_evolve(tmp_path)
    with pytest.raises(ValueError, match="expected 3"):
        write_hypotheses(3, layout=_layout(evolve), cycle_id=CycleId(1), adapter=StubLLMAdapter(n_lines=2))


def test_too_many_files_fails(tmp_path):
    evolve = _setup_evolve(tmp_path)
    with pytest.raises(ValueError, match="expected 3"):
        write_hypotheses(3, layout=_layout(evolve), cycle_id=CycleId(1), adapter=StubLLMAdapter(n_lines=4))


def test_no_files_fails(tmp_path):
    evolve = _setup_evolve(tmp_path)
    with pytest.raises(ValueError):
        write_hypotheses(2, layout=_layout(evolve), cycle_id=CycleId(1), adapter=StubLLMAdapter())


def test_adapter_nonzero_exit_propagates(tmp_path):
    evolve = _setup_evolve(tmp_path)
    with pytest.raises(subprocess.CalledProcessError):
        write_hypotheses(
            3,
            layout=_layout(evolve),
            cycle_id=CycleId(1),
            adapter=StubLLMAdapter(exit_code=5, n_lines=3),
        )


def test_cold_start_with_empty_ledger_and_no_leaderboard(tmp_path):
    evolve = _setup_evolve(tmp_path, with_ledger=False, with_leaderboard=False)
    files = write_hypotheses(2, layout=_layout(evolve), cycle_id=CycleId(1), adapter=StubLLMAdapter(n_lines=2))
    assert len(files) == 2


def test_adapter_invoked_with_expected_argv(tmp_path):
    evolve = _setup_evolve(tmp_path)
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(1),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    body = read_utf8(capture)
    assert "agent: hypotheses-writer" in body
    workspace = _layout(evolve).outputs.cycle(CycleId(1)).hypotheses_dir
    assert f"workspace: {workspace}" in body


def test_hypotheses_writer_passes_outputs_and_subject_as_readable_roots(tmp_path):
    evolve = _setup_evolve(tmp_path)
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(1),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    body = read_utf8(capture)
    layout = _layout(evolve)
    assert str(layout.outputs_root) in body
    assert str(layout.sources.subject_src) in body
    assert str(layout.sources.subject_dir) in body


def test_task_md_written_to_output_dir(tmp_path):
    evolve = _setup_evolve(tmp_path)
    _stage_prev_cycle(evolve, CycleId(6))
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(7),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    body = read_utf8(capture)
    assert "----- task.md -----" in body
    assert "0007" in body
    assert "Number of hypotheses: 2" in body


def test_returns_hypothesis_file_paths(tmp_path):
    evolve = _setup_evolve(tmp_path)
    cycle_layout = _layout(evolve).outputs.cycle(CycleId(1))
    files = write_hypotheses(2, layout=_layout(evolve), cycle_id=CycleId(1), adapter=StubLLMAdapter(n_lines=2))
    assert sorted(files) == [cycle_layout.hypothesis_file(HypothesisId(1)), cycle_layout.hypothesis_file(HypothesisId(2))]


@pytest.mark.slow
def test_cold_cycle_e2e_writes_real_hypothesis_files(tmp_path):
    repo_root = _copy_real_evolve(tmp_path)
    lines = [
        "Stash-bias eviction lifts hot-set hit rate on Zipfian traces.",
        "Lazy promotion cuts per-op time on read-heavy workloads.",
    ]
    # Stub overlay writes a `.overlay` sentinel, not a manifest impl file;
    # full-autonomy keeps it legal so this e2e run isn't scope-gated.
    cfg = load_configs(repo_root, cohort_size=2, authoring_scope="full-autonomy")
    run_cycle(
        layout=_layout(repo_root),
        repo_root=repo_root,
        adapter=StubLLMAdapter(hypothesis_lines=lines),
        cfg=cfg,
    )

    cycle_layout = _layout(repo_root).outputs.cycle(CycleId(1))
    h01 = read_utf8(cycle_layout.hypothesis_file(HypothesisId(1)))
    h02 = read_utf8(cycle_layout.hypothesis_file(HypothesisId(2)))
    assert "Stash-bias eviction" in h01
    assert "Lazy promotion" in h02
    assert "walking-skeleton stub" not in h01
    assert "walking-skeleton stub" not in h02
    assert h01 != h02


def test_cold_start_task_md_grounds_in_baseline(tmp_path):
    evolve = _setup_evolve(tmp_path)
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(1),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    task_md = read_utf8(capture)
    baseline = _layout(evolve).sources.baseline
    # Anchor on "baseline" so a stray "segment.rs" elsewhere doesn't satisfy
    # the check.
    assert "baseline" in task_md
    for path in (baseline.fitness_vector_json, baseline.insight_md):
        assert path.name in task_md, f"{path.name} missing from task.md"


def test_cold_start_task_md_omits_cross_cycle_stubs(tmp_path):
    evolve = _setup_evolve(tmp_path)
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(1),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    task_md = read_utf8(capture)
    layout = _layout(evolve)
    assert str(layout.outputs.leaderboard_md) not in task_md
    assert str(layout.outputs.ledger) not in task_md
    assert str(layout.outputs.cross_cycle_insights_md) not in task_md


def test_continuation_task_md_grounds_in_previous_cycle(tmp_path):
    evolve = _setup_evolve(tmp_path)
    _stage_prev_cycle(evolve, CycleId(1))
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(2),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    task_md = read_utf8(capture)
    prev = _layout(evolve).outputs.cycle(CycleId(1))
    # Anchor on "0001" so a leaked path from a different cycle wouldn't
    # satisfy the filename checks alone.
    assert "0001" in task_md
    for path in (prev.variants_dir, prev.cycle_winners_json, prev.cycle_insight_md):
        assert path.name in task_md, f"{path.name} missing from task.md"


def test_continuation_task_md_points_at_per_attempt_overlay(tmp_path):
    # A cycle variant's overlay lives under attempts/<NN>/overlay, not a flat
    # v<NNN>/overlay; the task.md must send the next hypothesizer to the real
    # location or it reads nothing.
    evolve = _setup_evolve(tmp_path)
    _stage_prev_cycle(evolve, CycleId(1))
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(2),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    task_md = read_utf8(capture)
    prev = _layout(evolve).outputs.cycle(CycleId(1))
    assert str(prev.variant_overlay_template) in task_md
    assert "attempts/<NN>/overlay" in task_md
    # The flat (wrong) overlay path must not be advertised.
    assert str(prev.variant_template.overlay_dir) not in task_md


def test_continuation_task_md_lists_baseline_as_valid_parent_variant(tmp_path):
    evolve = _setup_evolve(tmp_path)
    _stage_prev_cycle(evolve, CycleId(1))
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(2),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    task_md = read_utf8(capture)
    assert "baseline" in task_md


def test_continuation_task_md_lists_cross_cycle_artefacts(tmp_path):
    evolve = _setup_evolve(tmp_path)
    _stage_prev_cycle(evolve, CycleId(1))
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(2),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
    )
    task_md = read_utf8(capture)
    layout = _layout(evolve)
    assert layout.outputs.leaderboard_md.name in task_md
    assert layout.outputs.ledger.name in task_md
    assert layout.outputs.cross_cycle_insights_md.name in task_md


def test_continuation_fails_fast_when_prev_cycle_missing(tmp_path):
    evolve = _setup_evolve(tmp_path)
    with pytest.raises(FileNotFoundError, match="previous cycle"):
        write_hypotheses(
            2,
            layout=_layout(evolve),
            cycle_id=CycleId(2),
            adapter=StubLLMAdapter(n_lines=2),
        )


def _write_cross_cycle_winners(evolve: Path, *, winners: list[str], fitness_vectors: list[dict]) -> None:
    cross_cycle_winners = {
        "winners": winners,
        "fitness_vectors": fitness_vectors,
        "best_per_dimension": {},
    }
    cross_cycle_winners_json = _layout(evolve).outputs.cross_cycle_winners_json
    cross_cycle_winners_json.parent.mkdir(parents=True, exist_ok=True)
    write_utf8(cross_cycle_winners_json, json.dumps(cross_cycle_winners))


def _fitness_vector(variant: str, hit_rate: float) -> dict:
    return {
        "variant": variant,
        "dimensions": [{"dim": "hit_rate", "summary": hit_rate, "higher_is_better": True}],
    }


def test_continuation_injects_incumbents_and_quota(tmp_path):
    evolve = _setup_evolve(tmp_path)
    _stage_prev_cycle(evolve, CycleId(1))
    _write_cross_cycle_winners(
        evolve,
        winners=["baseline", "cycle/0001/v001"],
        fitness_vectors=[
            _fitness_vector("baseline", 0.40),
            _fitness_vector("cycle/0001/v001", 0.55),
        ],
    )
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(2),
        adapter=StubLLMAdapter(
            n_lines=2,
            capture_file=capture,
            parent_variants=["cycle/0001/v001", "baseline"],
        ),
    )
    task_md = read_utf8(capture)
    assert "Incumbents (cross-cycle winners)" in task_md
    assert "cycle/0001/v001: hit_rate=0.55" in task_md
    assert "MUST cite a cross-cycle winner" in task_md


def test_continuation_quota_violation_fails(tmp_path):
    evolve = _setup_evolve(tmp_path)
    _stage_prev_cycle(evolve, CycleId(1))
    _write_cross_cycle_winners(
        evolve,
        winners=["baseline", "cycle/0001/v001"],
        fitness_vectors=[
            _fitness_vector("baseline", 0.40),
            _fitness_vector("cycle/0001/v001", 0.55),
        ],
    )
    # Both hypotheses parent on baseline; none anchors the non-baseline front.
    with pytest.raises(ValueError, match="winner anchor"):
        write_hypotheses(
            2,
            layout=_layout(evolve),
            cycle_id=CycleId(2),
            adapter=StubLLMAdapter(n_lines=2, parent_variants=["baseline", "baseline"]),
        )


def test_continuation_quota_satisfied_by_front_anchor(tmp_path):
    evolve = _setup_evolve(tmp_path)
    _stage_prev_cycle(evolve, CycleId(1))
    _write_cross_cycle_winners(
        evolve,
        winners=["baseline", "cycle/0001/v001"],
        fitness_vectors=[
            _fitness_vector("baseline", 0.40),
            _fitness_vector("cycle/0001/v001", 0.55),
        ],
    )
    files = write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(2),
        adapter=StubLLMAdapter(n_lines=2, parent_variants=["cycle/0001/v001", "baseline"]),
    )
    assert len(files) == 2


def test_continuation_no_cross_cycle_winners_skips_quota(tmp_path):
    # No cross-cycle-winners.json: enforcement is skipped so a pure-baseline cohort passes.
    evolve = _setup_evolve(tmp_path)
    _stage_prev_cycle(evolve, CycleId(1))
    files = write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(2),
        adapter=StubLLMAdapter(n_lines=2, parent_variants=["baseline", "baseline"]),
    )
    assert len(files) == 2


def test_authoring_scope_section_lists_extension_points(tmp_path):
    evolve = _copy_real_evolve(tmp_path)
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(1),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
        authoring_scope=AuthoringScope.EXTENSION_POINTS,
    )
    task_md = read_utf8(capture)
    assert "## Authoring scope" in task_md
    assert "extension-point impl files" in task_md
    assert "writable by no scope" in task_md


def test_authoring_scope_section_full_autonomy(tmp_path):
    evolve = _copy_real_evolve(tmp_path)
    capture = tmp_path / "argv-capture.txt"
    write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(1),
        adapter=StubLLMAdapter(n_lines=2, capture_file=capture),
        authoring_scope=AuthoringScope.FULL_AUTONOMY,
    )
    task_md = read_utf8(capture)
    assert "## Authoring scope" in task_md
    assert "Any overlay file may be edited" in task_md


# The real dash manifest's extension-point impl files: eviction_policy.rs,
# dash_bucket.rs, item.rs, dash_segment.rs. `src/dash/segments/mod.rs` (home of
# SegmentCore) is the canonical out-of-scope file — a non-extension-point source
# the writer must steer changes away from (the run failure these tests guard
# against).


class _ScopeVaryingAdapter:
    """Hypotheses-writer stub returning a different `## Changes` per call, so the
    scope-regenerate loop can be driven end to end. Counts its invocations."""

    def __init__(self, change_per_call: list[str]) -> None:
        self._change_per_call = change_per_call
        self.calls = 0

    def identity(self) -> str:
        return "ScopeVaryingAdapter"

    def run(self, agent_name, workspace, *, readable_roots, allow_edits) -> None:
        change = self._change_per_call[min(self.calls, len(self._change_per_call) - 1)]
        self.calls += 1
        for i in (1, 2):
            write_utf8(
                workspace / f"H{i:02d}.md",
                f"# Hypothesis H{i:02d}\n\nclaim {i}\n\n## Parent\n\nbaseline\n\n## Changes\n\n{change}\n",
            )


def test_extension_scope_passes_when_changes_in_scope(tmp_path):
    evolve = _copy_real_evolve(tmp_path)
    files = write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(1),
        adapter=StubLLMAdapter(
            n_lines=2,
            changes=["Tighten eviction_policy.rs victim selection.", "Rebalance in dash_bucket.rs."],
        ),
        authoring_scope=AuthoringScope.EXTENSION_POINTS,
        attempts=2,
    )
    assert len(files) == 2


def test_extension_scope_regenerates_on_out_of_scope_then_succeeds(tmp_path):
    evolve = _copy_real_evolve(tmp_path)
    adapter = _ScopeVaryingAdapter(
        [
            "Thread InsertPosition through src/dash/segments/mod.rs.",  # out of scope
            "Adjust the candidate index in eviction_policy.rs.",  # back in scope
        ]
    )
    files = write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(1),
        adapter=adapter,
        authoring_scope=AuthoringScope.EXTENSION_POINTS,
        attempts=2,
    )
    assert adapter.calls == 2
    assert len(files) == 2
    task_md = read_utf8(_layout(evolve).outputs.cycle(CycleId(1)).hypotheses_task_md)
    assert "## Scope correction" in task_md
    assert "src/dash/segments/mod.rs" in task_md


def test_extension_scope_exhaustion_raises(tmp_path):
    evolve = _copy_real_evolve(tmp_path)
    adapter = _ScopeVaryingAdapter(["Thread it through src/dash/segments/mod.rs."])
    with pytest.raises(ValueError, match="outside the authoring scope"):
        write_hypotheses(
            2,
            layout=_layout(evolve),
            cycle_id=CycleId(1),
            adapter=adapter,
            authoring_scope=AuthoringScope.EXTENSION_POINTS,
            attempts=2,
        )
    assert adapter.calls == 2


def test_full_autonomy_skips_scope_check(tmp_path):
    evolve = _copy_real_evolve(tmp_path)
    files = write_hypotheses(
        2,
        layout=_layout(evolve),
        cycle_id=CycleId(1),
        adapter=StubLLMAdapter(
            n_lines=2,
            changes=["Edit src/dash/segments/mod.rs freely.", "Rework segment.rs."],
        ),
        authoring_scope=AuthoringScope.FULL_AUTONOMY,
        attempts=2,
    )
    assert len(files) == 2


def test_subagent_markdown_states_contract():
    agent_md = SourcesLayout(root=REPO_SRC).agent_prompt("hypotheses-writer")
    assert agent_md.is_file(), f"missing {agent_md}"
    body = read_utf8(agent_md)
    assert "hypothesis" in body.lower()
    assert "H<NN>.md" in body
    assert "N distinct" in body or "N Hypotheses" in body
    assert "## Parent" in body
    assert "## Changes" in body
    assert "baseline" in body and "cycle/<NNNN>/v<NNN>" in body
    # Scope limit: agent must not write outside the output dir.
    assert "outside" in body.lower() and "output" in body.lower()
