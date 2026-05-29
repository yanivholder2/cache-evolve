"""Tests for src/evolve/cycle/cross_cycle_insight_writer.write_cross_cycle_insight."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from evolve._io import append_utf8, read_utf8, write_utf8
from evolve.cycle.cross_cycle_insight_writer import (
    AGENT_NAME,
    OUTPUT_FILENAME,
    write_cross_cycle_insight,
)
from evolve.identifiers import CycleId
from evolve.paths import Layout

from _support import STUB_SUBJECT_NAME


OUTPUT_BODY = "# Cross-Cycle Insights\n\nstub\n"


def _layout(tmp_path: Path) -> Layout:
    evolve_root = tmp_path / "evolve"
    outputs_root = evolve_root / "outputs"
    outputs_root.mkdir(parents=True)
    return Layout(repo_root=evolve_root, outputs_root=outputs_root, subject=STUB_SUBJECT_NAME)


def _seed_ledger(ledger: Path, rows: list[tuple[str, str]]) -> None:
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
    """Captures the workspace task.md and writes a non-empty output file."""

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
        write_utf8(workspace / OUTPUT_FILENAME, OUTPUT_BODY)


def _cycle_layout(layout: Layout, cycle_id: CycleId):
    cycle = layout.outputs.cycle(cycle_id)
    cycle.root.mkdir(parents=True)
    return cycle


def test_task_md_inlines_full_ledger(tmp_path):
    layout = _layout(tmp_path)
    cycle_layout = _cycle_layout(layout, CycleId(3))
    _seed_ledger(
        layout.outputs.ledger,
        [
            ("0001", "first cycle body"),
            ("0002", "second cycle body"),
            ("0003", "third cycle body"),
        ],
    )

    adapter = _RecordingAdapter()
    write_cross_cycle_insight(
        cycle_layout,
        layout=layout,
        adapter=adapter,
    )

    task_md = adapter.calls[0]["task_md"]
    assert "### Cycle 0001" in task_md
    assert "first cycle body" in task_md
    assert "### Cycle 0002" in task_md
    assert "second cycle body" in task_md
    assert "### Cycle 0003" in task_md
    assert "third cycle body" in task_md
    assert layout.outputs.cross_cycle_insights_md.is_file()
    assert read_utf8(layout.outputs.cross_cycle_insights_md) == OUTPUT_BODY
    assert adapter.calls[0]["agent_name"] == AGENT_NAME


def test_task_md_has_section_template(tmp_path):
    layout = _layout(tmp_path)
    cycle_layout = _cycle_layout(layout, CycleId(1))
    _seed_ledger(layout.outputs.ledger, [("0001", "body")])

    adapter = _RecordingAdapter()
    write_cross_cycle_insight(cycle_layout, layout=layout, adapter=adapter)

    task_md = adapter.calls[0]["task_md"]
    for section in (
        "## Recurring wins",
        "## Recurring failures",
        "## Open questions",
        "## Per-policy notes",
    ):
        assert section in task_md, f"{section} missing from task.md"


def test_readable_roots_are_ledger_only(tmp_path):
    layout = _layout(tmp_path)
    cycle_layout = _cycle_layout(layout, CycleId(1))
    _seed_ledger(layout.outputs.ledger, [("0001", "body")])
    # Sibling artifacts that must NOT be reachable to the agent.
    layout.outputs.leaderboard_md.parent.mkdir(parents=True, exist_ok=True)
    write_utf8(layout.outputs.leaderboard_md, "leaderboard\n")
    write_utf8(layout.outputs.cross_cycle_insights_md, "previous file\n")

    adapter = _RecordingAdapter()
    write_cross_cycle_insight(cycle_layout, layout=layout, adapter=adapter)

    roots = adapter.calls[0]["readable_roots"]
    assert roots == [layout.outputs.ledger]
    assert adapter.calls[0]["allow_edits"] is True


def test_first_cycle_with_single_ledger_row(tmp_path):
    layout = _layout(tmp_path)
    cycle_layout = _cycle_layout(layout, CycleId(1))
    _seed_ledger(layout.outputs.ledger, [("0001", "only cycle so far")])

    adapter = _RecordingAdapter()
    write_cross_cycle_insight(cycle_layout, layout=layout, adapter=adapter)

    task_md = adapter.calls[0]["task_md"]
    assert "### Cycle 0001" in task_md
    assert "only cycle so far" in task_md
    assert layout.outputs.cross_cycle_insights_md.is_file()


class _OutputAdapter:
    """Adapter that writes a caller-provided payload (None = write nothing)."""

    def __init__(self, payload: str | None) -> None:
        self.payload = payload

    def identity(self) -> str:
        return "OutputAdapter"

    def run(self, agent_name, workspace, *, readable_roots, allow_edits) -> None:
        if self.payload is not None:
            write_utf8(workspace / OUTPUT_FILENAME, self.payload)


@pytest.mark.parametrize(
    "payload, exc, match",
    [
        (None, FileNotFoundError, OUTPUT_FILENAME),
        ("", ValueError, "empty"),
        ("   \n\t\n", ValueError, "empty"),
    ],
)
def test_invalid_agent_output_raises_and_preserves_prior_file(tmp_path, payload, exc, match):
    layout = _layout(tmp_path)
    cycle_layout = _cycle_layout(layout, CycleId(1))
    _seed_ledger(layout.outputs.ledger, [("0001", "body")])
    write_utf8(layout.outputs.cross_cycle_insights_md, "prior content\n")

    with pytest.raises(exc, match=match):
        write_cross_cycle_insight(
            cycle_layout,
            layout=layout,
            adapter=_OutputAdapter(payload),
        )
    assert read_utf8(layout.outputs.cross_cycle_insights_md) == "prior content\n"
