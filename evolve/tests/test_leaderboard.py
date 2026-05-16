"""Tests for scripts/leaderboard.py."""

import json
import subprocess
import sys
from pathlib import Path

SCRIPTS_DIR = Path(__file__).parent.parent / "scripts"
LEADERBOARD = SCRIPTS_DIR / "leaderboard.py"


def _run(evolve_root, output=None):
    cmd = [sys.executable, str(LEADERBOARD), "--evolve-root", str(evolve_root)]
    if output:
        cmd += ["--output", str(output)]
    return subprocess.run(cmd, capture_output=True, text=True)


def _setup_cycle(cycles_dir, cycle_id, variants):
    """Create a minimal cycle directory with winners.json."""
    cycle_dir = cycles_dir / cycle_id
    cycle_dir.mkdir(parents=True)
    winners = {
        "cycle": cycle_id,
        "worker_id": "local",
        "pareto_front": [v["variant"] for v in variants],
        "fitness_vectors": variants,
    }
    (cycle_dir / "winners.json").write_text(json.dumps(winners))
    return cycle_dir


def _make_evolve_root(tmp_path, cycles_data):
    evolve = tmp_path / "evolve"
    (evolve / "cycles").mkdir(parents=True)
    (evolve / "ledger").mkdir()
    ledger = evolve / "ledger" / "cycles.jsonl"
    for cycle_id, variants in cycles_data.items():
        _setup_cycle(evolve / "cycles", cycle_id, variants)
        ledger.open("a").write(json.dumps({"cycle": cycle_id, "worker_id": "local", "cycle_dir": f"evolve/cycles/{cycle_id}"}) + "\n")
    return evolve


def test_generates_leaderboard_md(tmp_path):
    fv = [{"variant": "v001", "dimensions": [{"dim": "hit-rate", "summary": 0.65, "higher_is_better": True}]}]
    evolve = _make_evolve_root(tmp_path, {"0001": fv})
    out = tmp_path / "leaderboard.md"

    result = _run(evolve, output=out)
    assert result.returncode == 0, result.stderr
    content = out.read_text()
    assert "leaderboard" in content.lower()
    assert "0001" in content
    assert "v001" in content


def test_empty_ledger_produces_valid_md(tmp_path):
    evolve = tmp_path / "evolve"
    (evolve / "cycles").mkdir(parents=True)
    (evolve / "ledger").mkdir()
    (evolve / "ledger" / "cycles.jsonl").write_text("")

    out = tmp_path / "leaderboard.md"
    result = _run(evolve, output=out)
    assert result.returncode == 0, result.stderr
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
    result = _run(evolve, output=out)
    assert result.returncode == 0, result.stderr
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
    result = _run(evolve, output=out)
    assert result.returncode == 0, result.stderr
    content = out.read_text()
    assert "v001" in content
    assert "v002" in content
    # Both summaries should appear comma-joined in the same row
    assert "0.65" in content and "0.7" in content


def test_pareto_front_fallback_renders(tmp_path):
    """winners.json with empty fitness_vectors falls back to pareto_front for variant names."""
    evolve = tmp_path / "evolve"
    (evolve / "cycles" / "0001").mkdir(parents=True)
    (evolve / "ledger").mkdir()
    (evolve / "ledger" / "cycles.jsonl").write_text(
        json.dumps({"cycle": "0001", "worker_id": "local", "cycle_dir": "evolve/cycles/0001"}) + "\n"
    )
    (evolve / "cycles" / "0001" / "winners.json").write_text(json.dumps({
        "cycle": "0001",
        "worker_id": "local",
        "pareto_front": ["v007"],
        "fitness_vectors": [],
    }))
    out = tmp_path / "leaderboard.md"
    result = _run(evolve, output=out)
    assert result.returncode == 0, result.stderr
    assert "v007" in out.read_text()


def test_malformed_ledger_errors(tmp_path):
    evolve = tmp_path / "evolve"
    (evolve / "cycles").mkdir(parents=True)
    (evolve / "ledger").mkdir()
    (evolve / "ledger" / "cycles.jsonl").write_text('{"cycle": "0001"}\nnot json\n')
    result = _run(evolve)
    assert result.returncode != 0
    assert "malformed ledger" in result.stderr


def test_malformed_winners_errors(tmp_path):
    evolve = tmp_path / "evolve"
    (evolve / "cycles" / "0001").mkdir(parents=True)
    (evolve / "ledger").mkdir()
    (evolve / "ledger" / "cycles.jsonl").write_text(
        json.dumps({"cycle": "0001", "worker_id": "local", "cycle_dir": "evolve/cycles/0001"}) + "\n"
    )
    (evolve / "cycles" / "0001" / "winners.json").write_text("not valid json {{{")
    result = _run(evolve)
    assert result.returncode != 0
    assert "malformed winners.json" in result.stderr


def test_multiple_cycles_all_shown(tmp_path):
    fv1 = [{"variant": "v001", "dimensions": [{"dim": "hit-rate", "summary": 0.65, "higher_is_better": True}]}]
    fv2 = [{"variant": "v002", "dimensions": [{"dim": "hit-rate", "summary": 0.70, "higher_is_better": True}]}]
    evolve = _make_evolve_root(tmp_path, {"0001": fv1, "0002": fv2})
    out = tmp_path / "leaderboard.md"

    result = _run(evolve, output=out)
    assert result.returncode == 0, result.stderr
    content = out.read_text()
    assert "0001" in content
    assert "0002" in content
