"""Tests for src/leaderboard.regenerate."""

import json

import pytest

from src._io import append_utf8, read_utf8, write_utf8
from src.identifiers import CycleId
from src.cycle.leaderboard import LeaderboardError, regenerate
from src.paths import OutputsLayout


def _setup_cycle(layout: OutputsLayout, cycle_id: CycleId, variants):
    cycle_layout = layout.cycle(cycle_id)
    cycle_layout.root.mkdir(parents=True)
    winners = {
        "cycle": str(cycle_id),
        "pareto_front": [v["variant"] for v in variants],
        "fitness_vectors": variants,
    }
    write_utf8(cycle_layout.winners_json, json.dumps(winners))
    return cycle_layout


def _make_evolve_root(tmp_path, cycles_data):
    evolve = tmp_path / "evolve"
    layout = OutputsLayout(root=evolve)
    layout.cycles_dir.mkdir(parents=True)
    layout.ledger.parent.mkdir()
    for cycle_str, variants in cycles_data.items():
        cycle_id = CycleId.from_str(cycle_str)
        _setup_cycle(layout, cycle_id, variants)
        append_utf8(layout.ledger, json.dumps({"cycle": str(cycle_id), "cycle_dir": f"evolve/cycles/{cycle_id}"}) + "\n")
    return evolve


def test_generates_leaderboard_md(tmp_path):
    fv = [{"variant": "v001", "dimensions": [{"dim": "hit-rate", "summary": 0.65, "higher_is_better": True}]}]
    evolve = _make_evolve_root(tmp_path, {"0001": fv})
    out = tmp_path / "leaderboard.md"

    regenerate(layout=OutputsLayout(root=evolve), output=out)
    content = read_utf8(out)
    assert "leaderboard" in content.lower()
    assert "0001" in content
    assert "v001" in content


def test_empty_ledger_produces_valid_md(tmp_path):
    evolve = tmp_path / "evolve"
    layout = OutputsLayout(root=evolve)
    layout.cycles_dir.mkdir(parents=True)
    layout.ledger.parent.mkdir()
    write_utf8(layout.ledger, "")

    out = tmp_path / "leaderboard.md"
    regenerate(layout=layout, output=out)
    assert read_utf8(out)  # not empty


def test_multi_dimension_rendering(tmp_path):
    fv = [{
        "variant": "v001",
        "dimensions": [
            {"dim": "hit-rate", "summary": 0.65, "higher_is_better": True},
            {"dim": "per-op-time", "summary": 95.0, "higher_is_better": False},
        ],
    }]
    evolve = _make_evolve_root(tmp_path, {"0001": fv})
    out = tmp_path / "leaderboard.md"
    regenerate(layout=OutputsLayout(root=evolve), output=out)
    content = read_utf8(out)
    assert "hit-rate" in content
    assert "per-op-time" in content


def test_multi_variant_rendering(tmp_path):
    fv = [
        {"variant": "v001", "dimensions": [{"dim": "hit-rate", "summary": 0.65, "higher_is_better": True}]},
        {"variant": "v002", "dimensions": [{"dim": "hit-rate", "summary": 0.70, "higher_is_better": True}]},
    ]
    evolve = _make_evolve_root(tmp_path, {"0001": fv})
    out = tmp_path / "leaderboard.md"
    regenerate(layout=OutputsLayout(root=evolve), output=out)
    content = read_utf8(out)
    assert "v001" in content
    assert "v002" in content
    assert "0.65" in content and "0.7" in content


def test_pareto_front_fallback_renders(tmp_path):
    evolve = tmp_path / "evolve"
    layout = OutputsLayout(root=evolve)
    cycle_layout = layout.cycle(CycleId(1))
    cycle_layout.root.mkdir(parents=True)
    layout.ledger.parent.mkdir()
    write_utf8(
        layout.ledger,
        json.dumps({"cycle": "0001", "cycle_dir": "evolve/cycles/0001"}) + "\n",
    )
    write_utf8(cycle_layout.winners_json, json.dumps({
        "cycle": "0001",
        "pareto_front": ["v007"],
        "fitness_vectors": [],
    }))
    out = tmp_path / "leaderboard.md"
    regenerate(layout=layout, output=out)
    assert "v007" in read_utf8(out)


def test_malformed_ledger_errors(tmp_path):
    evolve = tmp_path / "evolve"
    layout = OutputsLayout(root=evolve)
    layout.cycles_dir.mkdir(parents=True)
    layout.ledger.parent.mkdir()
    write_utf8(layout.ledger, '{"cycle": "0001"}\nnot json\n')
    with pytest.raises(LeaderboardError, match="malformed ledger"):
        regenerate(layout=layout)


def test_malformed_winners_errors(tmp_path):
    evolve = tmp_path / "evolve"
    layout = OutputsLayout(root=evolve)
    cycle_layout = layout.cycle(CycleId(1))
    cycle_layout.root.mkdir(parents=True)
    layout.ledger.parent.mkdir()
    write_utf8(
        layout.ledger,
        json.dumps({"cycle": "0001", "cycle_dir": "evolve/cycles/0001"}) + "\n",
    )
    write_utf8(cycle_layout.winners_json, "not valid json {{{")
    with pytest.raises(LeaderboardError, match="malformed winners.json"):
        regenerate(layout=layout)


def test_multiple_cycles_all_shown(tmp_path):
    fv1 = [{"variant": "v001", "dimensions": [{"dim": "hit-rate", "summary": 0.65, "higher_is_better": True}]}]
    fv2 = [{"variant": "v002", "dimensions": [{"dim": "hit-rate", "summary": 0.70, "higher_is_better": True}]}]
    evolve = _make_evolve_root(tmp_path, {"0001": fv1, "0002": fv2})
    out = tmp_path / "leaderboard.md"

    regenerate(layout=OutputsLayout(root=evolve), output=out)
    content = read_utf8(out)
    assert "0001" in content
    assert "0002" in content
