"""Tests for src/evolve/cycle/cycle_insight_writer.write_cycle_insight."""

from __future__ import annotations

from pathlib import Path

import pytest

from evolve._io import append_utf8, read_utf8, write_utf8
from evolve.cycle.cycle_insight_writer import AGENT_NAME, write_cycle_insight
from evolve.identifiers import CycleId, HypothesisId, VariantId
from evolve.paths import CycleLayout


def _seed_variant(cycle_layout: CycleLayout, variant_id: VariantId, insight_body: str):
    variant_dir = cycle_layout.variant(variant_id)
    variant_dir.root.mkdir(parents=True, exist_ok=True)
    write_utf8(variant_dir.insight_md, insight_body)
    return variant_dir


def _seed_hypothesis(cycle_layout: CycleLayout, hypothesis_id: HypothesisId, body: str):
    cycle_layout.hypotheses_dir.mkdir(parents=True, exist_ok=True)
    path = cycle_layout.hypothesis_file(hypothesis_id)
    write_utf8(path, body)
    return path


def _seed_ledger(ledger: Path, rows: list[tuple[str, str]]) -> None:
    import json
    ledger.parent.mkdir(parents=True, exist_ok=True)
    for cycle, body in rows:
        append_utf8(
            ledger,
            json.dumps({
                "cycle": cycle,
                "cycle_dir": f"cycles/{cycle}",
                "cycle_insight": body,
            }) + "\n",
        )


class _RecordingAdapter:
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
        write_utf8(workspace / "cycle-insight.md", "# Cycle Insight\n\nstub\n")


class _SilentAdapter:
    def identity(self) -> str:
        return "SilentAdapter"

    def run(self, agent_name, workspace, *, readable_roots, allow_edits) -> None:
        return None


def _cycle_layout(tmp_path: Path, cycle_id: CycleId) -> CycleLayout:
    cycle_layout = CycleLayout(root=tmp_path / "outputs" / "cycles" / str(cycle_id))
    cycle_layout.root.mkdir(parents=True)
    return cycle_layout


def test_task_md_contains_pareto_insights_and_hypotheses(tmp_path):
    cycle_layout = _cycle_layout(tmp_path, CycleId(1))
    v1 = _seed_variant(cycle_layout, VariantId(1), "v001 insight body\n")
    v2 = _seed_variant(cycle_layout, VariantId(2), "v002 insight body\n")
    h1 = _seed_hypothesis(cycle_layout, HypothesisId(1), "h1 body\n")
    h2 = _seed_hypothesis(cycle_layout, HypothesisId(2), "h2 body\n")
    ledger = tmp_path / "cycles-ledger.jsonl"

    adapter = _RecordingAdapter()
    write_cycle_insight(
        cycle_layout,
        survivors=[v1, v2],
        pareto_front=["v002", "v001"],
        hypothesis_files=[h1, h2],
        ledger=ledger,
        adapter=adapter,
        digest_cycles=3,
    )

    task_md = adapter.calls[0]["task_md"]
    assert "v002" in task_md
    assert "v001" in task_md
    assert str(v1.insight_md) in task_md
    assert str(v2.insight_md) in task_md
    assert str(h1) in task_md
    assert str(h2) in task_md


def test_task_md_inlines_digest_body(tmp_path):
    cycle_layout = _cycle_layout(tmp_path, CycleId(2))
    v1 = _seed_variant(cycle_layout, VariantId(1), "vbody")
    h1 = _seed_hypothesis(cycle_layout, HypothesisId(1), "h body")
    ledger = tmp_path / "cycles-ledger.jsonl"
    _seed_ledger(ledger, [("0001", "prior cycle continuation text")])

    adapter = _RecordingAdapter()
    write_cycle_insight(
        cycle_layout,
        survivors=[v1],
        pareto_front=["v001"],
        hypothesis_files=[h1],
        ledger=ledger,
        adapter=adapter,
        digest_cycles=3,
    )

    task_md = adapter.calls[0]["task_md"]
    assert "### Cycle 0001" in task_md
    assert "prior cycle continuation text" in task_md


def test_readable_roots_are_insights_and_hypotheses_only(tmp_path):
    cycle_layout = _cycle_layout(tmp_path, CycleId(1))
    v1 = _seed_variant(cycle_layout, VariantId(1), "v1")
    v2 = _seed_variant(cycle_layout, VariantId(2), "v2")
    h1 = _seed_hypothesis(cycle_layout, HypothesisId(1), "h1")
    h2 = _seed_hypothesis(cycle_layout, HypothesisId(2), "h2")
    ledger = tmp_path / "cycles-ledger.jsonl"

    adapter = _RecordingAdapter()
    write_cycle_insight(
        cycle_layout,
        survivors=[v1, v2],
        pareto_front=["v001", "v002"],
        hypothesis_files=[h1, h2],
        ledger=ledger,
        adapter=adapter,
        digest_cycles=3,
    )

    roots = adapter.calls[0]["readable_roots"]
    assert v1.insight_md in roots
    assert v2.insight_md in roots
    assert h1 in roots
    assert h2 in roots
    assert v1.root not in roots
    assert v2.root not in roots
    assert v1.segment_rs not in roots
    assert adapter.calls[0]["agent_name"] == AGENT_NAME
    assert adapter.calls[0]["allow_edits"] is True


def test_missing_cycle_insight_md_raises(tmp_path):
    cycle_layout = _cycle_layout(tmp_path, CycleId(1))
    v1 = _seed_variant(cycle_layout, VariantId(1), "v1")
    h1 = _seed_hypothesis(cycle_layout, HypothesisId(1), "h1")
    ledger = tmp_path / "cycles-ledger.jsonl"

    with pytest.raises(FileNotFoundError, match="cycle-insight.md"):
        write_cycle_insight(
            cycle_layout,
            survivors=[v1],
            pareto_front=["v001"],
            hypothesis_files=[h1],
            ledger=ledger,
            adapter=_SilentAdapter(),
            digest_cycles=3,
        )


def test_no_prior_cycles_yields_no_digest_section_body(tmp_path):
    cycle_layout = _cycle_layout(tmp_path, CycleId(1))
    v1 = _seed_variant(cycle_layout, VariantId(1), "v1")
    h1 = _seed_hypothesis(cycle_layout, HypothesisId(1), "h1")
    ledger = tmp_path / "cycles-ledger.jsonl"

    adapter = _RecordingAdapter()
    write_cycle_insight(
        cycle_layout,
        survivors=[v1],
        pareto_front=["v001"],
        hypothesis_files=[h1],
        ledger=ledger,
        adapter=adapter,
        digest_cycles=3,
    )

    task_md = adapter.calls[0]["task_md"]
    assert "### Cycle" not in task_md
    assert "## Prior cycles digest\n\n## Write" in task_md
