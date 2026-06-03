"""Tests for src/cycle/leaderboard.regenerate."""

import json

import pytest

from evolve._io import read_utf8, write_utf8
from evolve.identifiers import CycleId
from evolve.cycle.leaderboard import LeaderboardError, regenerate
from evolve.paths import OutputsLayout


def _setup_cycle(layout: OutputsLayout, cycle_id: CycleId, variants):
    cycle_layout = layout.cycle(cycle_id)
    cycle_layout.root.mkdir(parents=True)
    winners = {
        "cycle": str(cycle_id),
        "winners": [v["variant"] for v in variants],
        "fitness_vectors": variants,
    }
    write_utf8(cycle_layout.cycle_winners_json, json.dumps(winners))
    return cycle_layout


def _make_evolve_root(tmp_path, cycles_data):
    evolve = tmp_path / "evolve"
    layout = OutputsLayout(root=evolve)
    layout.cycles_dir.mkdir(parents=True)
    for cycle_str, variants in cycles_data.items():
        cycle_id = CycleId.from_str(cycle_str)
        _setup_cycle(layout, cycle_id, variants)
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


def test_no_cycles_produces_valid_md(tmp_path):
    evolve = tmp_path / "evolve"
    layout = OutputsLayout(root=evolve)
    layout.cycles_dir.mkdir(parents=True)

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
    write_utf8(cycle_layout.cycle_winners_json, json.dumps({
        "cycle": "0001",
        "winners": ["v007"],
        "fitness_vectors": [],
    }))
    out = tmp_path / "leaderboard.md"
    regenerate(layout=layout, output=out)
    assert "v007" in read_utf8(out)


def test_malformed_winners_errors(tmp_path):
    evolve = tmp_path / "evolve"
    layout = OutputsLayout(root=evolve)
    cycle_layout = layout.cycle(CycleId(1))
    cycle_layout.root.mkdir(parents=True)
    write_utf8(cycle_layout.cycle_winners_json, "not valid json {{{")
    with pytest.raises(LeaderboardError, match="malformed cycle-winners.json"):
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


def test_unsealed_cycle_with_winners_is_included(tmp_path):
    """Cycle dirs with cycle-winners.json but no SEALED marker are still counted."""
    fv = [{"variant": "v001", "dimensions": [{"dim": "hit-rate", "summary": 0.5, "higher_is_better": True}]}]
    evolve = _make_evolve_root(tmp_path, {"0001": fv})
    # No SEALED marker written.
    layout = OutputsLayout(root=evolve)
    assert not layout.cycle(CycleId(1)).sealed_marker.exists()

    out = tmp_path / "leaderboard.md"
    regenerate(layout=layout, output=out)
    assert "0001" in read_utf8(out)


def test_dir_without_winners_json_is_skipped(tmp_path):
    evolve = tmp_path / "evolve"
    layout = OutputsLayout(root=evolve)
    layout.cycles_dir.mkdir(parents=True)
    (layout.cycles_dir / "0001").mkdir()

    out = tmp_path / "leaderboard.md"
    regenerate(layout=layout, output=out)
    assert "0001" not in read_utf8(out)


def _write_cross_cycle_winners(evolve, cross_cycle_winners):
    write_utf8(OutputsLayout(root=evolve).cross_cycle_winners_json, json.dumps(cross_cycle_winners))


def test_baseline_row_and_global_best_section(tmp_path):
    fv = [{"variant": "v001", "dimensions": [{"dim": "hit-rate", "summary": 0.65, "higher_is_better": True}]}]
    evolve = _make_evolve_root(tmp_path, {"0001": fv})
    _write_cross_cycle_winners(evolve, {
        "winners": ["baseline", "cycle/0001/v001"],
        "fitness_vectors": [
            {"variant": "baseline", "dimensions": [{"dim": "hit-rate", "summary": 0.48, "higher_is_better": True}]},
            {"variant": "cycle/0001/v001", "dimensions": [{"dim": "hit-rate", "summary": 0.65, "higher_is_better": True}]},
        ],
        "best_per_dimension": {"hit-rate": {"variant": "cycle/0001/v001", "summary": 0.65}},
    })
    out = tmp_path / "leaderboard.md"
    regenerate(layout=OutputsLayout(root=evolve), output=out)
    content = read_utf8(out)

    # (a) baseline row
    assert "| baseline | baseline | 0.48 |" in content
    # (b) global best-so-far section
    assert "## Global best-so-far" in content
    assert "Pareto front: baseline, cycle/0001/v001" in content
    assert "- hit-rate: 0.65 held by cycle/0001/v001" in content


def test_dominated_marker(tmp_path):
    fv1 = [{"variant": "v001", "dimensions": [{"dim": "hit-rate", "summary": 0.65, "higher_is_better": True}]}]
    fv2 = [{"variant": "v001", "dimensions": [{"dim": "hit-rate", "summary": 0.40, "higher_is_better": True}]}]
    evolve = _make_evolve_root(tmp_path, {"0001": fv1, "0002": fv2})
    _write_cross_cycle_winners(evolve, {
        # cycle/0001/v001 on the front; cycle/0002/v001 dominated, off the front
        "winners": ["baseline", "cycle/0001/v001"],
        "fitness_vectors": [
            {"variant": "baseline", "dimensions": [{"dim": "hit-rate", "summary": 0.48, "higher_is_better": True}]},
            {"variant": "cycle/0001/v001", "dimensions": [{"dim": "hit-rate", "summary": 0.65, "higher_is_better": True}]},
        ],
        "best_per_dimension": {"hit-rate": {"variant": "cycle/0001/v001", "summary": 0.65}},
    })
    out = tmp_path / "leaderboard.md"
    regenerate(layout=OutputsLayout(root=evolve), output=out)
    content = read_utf8(out)

    # (c) off-front winner marked dominated; on-front winner not marked
    assert "| 0002 | v001 (dominated) |" in content
    assert "| 0001 | v001 |" in content


def test_malformed_cross_cycle_winners_errors(tmp_path):
    fv = [{"variant": "v001", "dimensions": [{"dim": "hit-rate", "summary": 0.65, "higher_is_better": True}]}]
    evolve = _make_evolve_root(tmp_path, {"0001": fv})
    write_utf8(OutputsLayout(root=evolve).cross_cycle_winners_json, "not valid json {{{")
    with pytest.raises(LeaderboardError, match="malformed cross-cycle-winners.json"):
        regenerate(layout=OutputsLayout(root=evolve))


def test_baseline_row_skipped_when_no_baseline_vector(tmp_path):
    fv = [{"variant": "v001", "dimensions": [{"dim": "hit-rate", "summary": 0.65, "higher_is_better": True}]}]
    evolve = _make_evolve_root(tmp_path, {"0001": fv})
    _write_cross_cycle_winners(evolve, {
        "winners": ["cycle/0001/v001"],
        "fitness_vectors": [
            {"variant": "cycle/0001/v001", "dimensions": [{"dim": "hit-rate", "summary": 0.65, "higher_is_better": True}]},
        ],
        "best_per_dimension": {"hit-rate": {"variant": "cycle/0001/v001", "summary": 0.65}},
    })
    out = tmp_path / "leaderboard.md"
    regenerate(layout=OutputsLayout(root=evolve), output=out)
    content = read_utf8(out)
    assert "| baseline | baseline" not in content
    assert "## Global best-so-far" in content


def test_non_cycle_subdirs_skipped(tmp_path):
    fv = [{"variant": "v001", "dimensions": [{"dim": "hit-rate", "summary": 0.5, "higher_is_better": True}]}]
    evolve = _make_evolve_root(tmp_path, {"0001": fv})
    layout = OutputsLayout(root=evolve)
    (layout.cycles_dir / "junk").mkdir()
    (layout.cycles_dir / "12345").mkdir()  # wrong width

    out = tmp_path / "leaderboard.md"
    regenerate(layout=layout, output=out)
    content = read_utf8(out)
    assert "0001" in content
    assert "junk" not in content
    assert "12345" not in content
