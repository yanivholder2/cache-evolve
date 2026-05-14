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
