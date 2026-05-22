"""Tests for src/leaderboard.regenerate."""

import json

import pytest

from src.leaderboard import LeaderboardError, regenerate
from src.paths import OutputsLayout


def _setup_cycle(layout: OutputsLayout, cycle_id, variants):
    """Create a minimal cycle directory with winners.json."""

    cycle_layout = layout.cycle(cycle_id)
    cycle_layout.root.mkdir(parents=True)
    winners = {
        "cycle": cycle_id,
        "pareto_front": [v["variant"] for v in variants],
        "fitness_vectors": variants,
    }
    cycle_layout.winners_json.write_text(json.dumps(winners))
    return cycle_layout


def _make_evolve_root(tmp_path, cycles_data):
    evolve = tmp_path / "evolve"
    layout = OutputsLayout(root=evolve)
    layout.cycles_dir.mkdir(parents=True)
    layout.ledger.parent.mkdir()
    for cycle_id, variants in cycles_data.items():
        _setup_cycle(layout, cycle_id, variants)
        layout.ledger.open("a").write(json.dumps({"cycle": cycle_id, "cycle_dir": f"evolve/cycles/{cycle_id}"}) + "\n")
    return evolve


def test_generates_leaderboard_md(tmp_path):
    fv = [{"variant": "v001", "dimensions": [{"dim": "hit-rate", "summary": 0.65, "higher_is_better": True}]}]
    evolve = _make_evolve_root(tmp_path, {"0001": fv})
    out = tmp_path / "leaderboard.md"

    regenerate(layout=OutputsLayout(root=evolve), output=out)
    content = out.read_text()
    assert "leaderboard" in content.lower()
    assert "0001" in content
    assert "v001" in content


def test_empty_ledger_produces_valid_md(tmp_path):
    evolve = tmp_path / "evolve"
    layout = OutputsLayout(root=evolve)
    layout.cycles_dir.mkdir(parents=True)
    layout.ledger.parent.mkdir()
    layout.ledger.write_text("")

    out = tmp_path / "leaderboard.md"
    regenerate(layout=layout, output=out)
    assert out.read_text()  # not empty


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
    content = out.read_text()
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
    content = out.read_text()
    assert "v001" in content
    assert "v002" in content
    # Both summaries should appear comma-joined in the same row
    assert "0.65" in content and "0.7" in content


def test_pareto_front_fallback_renders(tmp_path):
    """winners.json with empty fitness_vectors falls back to pareto_front for variant names."""

    evolve = tmp_path / "evolve"
    layout = OutputsLayout(root=evolve)
    cycle_layout = layout.cycle("0001")
    cycle_layout.root.mkdir(parents=True)
    layout.ledger.parent.mkdir()
    layout.ledger.write_text(
        json.dumps({"cycle": "0001", "cycle_dir": "evolve/cycles/0001"}) + "\n"
    )
    cycle_layout.winners_json.write_text(json.dumps({
        "cycle": "0001",
        "pareto_front": ["v007"],
        "fitness_vectors": [],
    }))
    out = tmp_path / "leaderboard.md"
    regenerate(layout=layout, output=out)
    assert "v007" in out.read_text()


def test_malformed_ledger_errors(tmp_path):
    evolve = tmp_path / "evolve"
    layout = OutputsLayout(root=evolve)
    layout.cycles_dir.mkdir(parents=True)
    layout.ledger.parent.mkdir()
    layout.ledger.write_text('{"cycle": "0001"}\nnot json\n')
    with pytest.raises(LeaderboardError, match="malformed ledger"):
        regenerate(layout=layout)


def test_malformed_winners_errors(tmp_path):
    evolve = tmp_path / "evolve"
    layout = OutputsLayout(root=evolve)
    cycle_layout = layout.cycle("0001")
    cycle_layout.root.mkdir(parents=True)
    layout.ledger.parent.mkdir()
    layout.ledger.write_text(
        json.dumps({"cycle": "0001", "cycle_dir": "evolve/cycles/0001"}) + "\n"
    )
    cycle_layout.winners_json.write_text("not valid json {{{")
    with pytest.raises(LeaderboardError, match="malformed winners.json"):
        regenerate(layout=layout)


def test_multiple_cycles_all_shown(tmp_path):
    fv1 = [{"variant": "v001", "dimensions": [{"dim": "hit-rate", "summary": 0.65, "higher_is_better": True}]}]
    fv2 = [{"variant": "v002", "dimensions": [{"dim": "hit-rate", "summary": 0.70, "higher_is_better": True}]}]
    evolve = _make_evolve_root(tmp_path, {"0001": fv1, "0002": fv2})
    out = tmp_path / "leaderboard.md"

    regenerate(layout=OutputsLayout(root=evolve), output=out)
    content = out.read_text()
    assert "0001" in content
    assert "0002" in content
