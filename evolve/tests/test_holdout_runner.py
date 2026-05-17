"""Held-out runner contracts.

The runner is the only path by which holdout traces flow into the cycle
artifacts. These tests pin:
- it reads the cycle's Pareto front from winners.json
- it invokes the hit-rate plugin with TRACE_SUITE=holdout
- it writes holdout.json with per-variant deltas and an any_regression flag
- regression detection respects REGRESSION_THRESHOLD
"""

import json
import os
import subprocess
import sys
from pathlib import Path
from unittest import mock

EVOLVE_ROOT = Path(__file__).parent.parent
SCRIPTS_DIR = EVOLVE_ROOT / "scripts"
RUNNER = SCRIPTS_DIR / "holdout-runner.py"

sys.path.insert(0, str(SCRIPTS_DIR))
import importlib.util

_spec = importlib.util.spec_from_file_location("holdout_runner", RUNNER)
_holdout_runner = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_holdout_runner)  # type: ignore[union-attr]


def _scaffold_cycle(tmp_path: Path, dev_summary: float) -> Path:
    cycle = tmp_path / "cycles" / "0001"
    (cycle / "variants" / "v001" / "eval").mkdir(parents=True)
    (cycle / "winners.json").write_text(json.dumps({
        "cycle": "0001",
        "worker_id": "test",
        "pareto_front": ["v001"],
    }))
    fv = {
        "variant": "v001",
        "dimensions": [
            {"dim": "hit-rate", "values": [], "summary": dev_summary, "higher_is_better": True},
        ],
    }
    (cycle / "variants" / "v001" / "eval" / "fitness-vector.json").write_text(json.dumps(fv))
    return cycle


def test_runner_flags_regression_when_holdout_drops(tmp_path):
    cycle = _scaffold_cycle(tmp_path, dev_summary=0.80)

    # Stub the plugin so we don't have to build/run the Rust binary in unit tests.
    stub_holdout = {"dim": "hit-rate", "values": [{"trace": "x", "cache": "dash", "size": 32, "hit_rate": 0.70}], "summary": 0.70, "higher_is_better": True, "suite": "holdout"}
    with mock.patch.object(_holdout_runner, "_run_holdout_plugin", return_value=stub_holdout):
        result = _holdout_runner.evaluate_holdout(cycle, repo_root=Path("."), evolve_root=EVOLVE_ROOT, threshold=0.02)

    assert result["any_regression"] is True
    assert result["variants"][0]["regression"] is True
    assert result["variants"][0]["delta"] < 0
    assert (cycle / "holdout.json").is_file()
    written = json.loads((cycle / "holdout.json").read_text())
    assert written["any_regression"] is True


def test_runner_does_not_flag_when_within_threshold(tmp_path):
    cycle = _scaffold_cycle(tmp_path, dev_summary=0.80)

    stub_holdout = {"dim": "hit-rate", "values": [], "summary": 0.79, "higher_is_better": True, "suite": "holdout"}
    with mock.patch.object(_holdout_runner, "_run_holdout_plugin", return_value=stub_holdout):
        result = _holdout_runner.evaluate_holdout(cycle, repo_root=Path("."), evolve_root=EVOLVE_ROOT, threshold=0.02)

    assert result["any_regression"] is False
    assert result["variants"][0]["regression"] is False


def test_runner_uses_holdout_suite_env_var(tmp_path, monkeypatch):
    """The runner must invoke the hit-rate plugin with TRACE_SUITE=holdout."""
    cycle = _scaffold_cycle(tmp_path, dev_summary=0.50)

    captured_env = {}

    def fake_run(cmd, capture_output, text, env):
        captured_env.update(env)
        return subprocess.CompletedProcess(cmd, 0, stdout=json.dumps({
            "dim": "hit-rate", "values": [], "summary": 0.55, "higher_is_better": True, "suite": "holdout",
        }), stderr="")

    with mock.patch.object(subprocess, "run", side_effect=fake_run):
        _holdout_runner.evaluate_holdout(cycle, repo_root=Path("."), evolve_root=EVOLVE_ROOT, threshold=0.02)

    assert captured_env.get("TRACE_SUITE") == "holdout"


def test_holdout_is_never_invoked_for_non_pareto_variants(tmp_path):
    """Sanity: variants outside the Pareto front are not in winners.json, so the runner
    must not touch their fitness vectors."""
    cycle = tmp_path / "cycles" / "0001"
    (cycle / "variants" / "v001" / "eval").mkdir(parents=True)
    (cycle / "variants" / "v002" / "eval").mkdir(parents=True)
    (cycle / "winners.json").write_text(json.dumps({
        "cycle": "0001",
        "worker_id": "test",
        "pareto_front": ["v001"],  # v002 dominated, must be skipped
    }))
    fv = {"variant": "v001", "dimensions": [{"dim": "hit-rate", "values": [], "summary": 0.6, "higher_is_better": True}]}
    (cycle / "variants" / "v001" / "eval" / "fitness-vector.json").write_text(json.dumps(fv))
    # v002 deliberately has no fitness-vector.json — runner must not read it

    stub_holdout = {"dim": "hit-rate", "values": [], "summary": 0.6, "higher_is_better": True}
    with mock.patch.object(_holdout_runner, "_run_holdout_plugin", return_value=stub_holdout):
        result = _holdout_runner.evaluate_holdout(cycle, repo_root=Path("."), evolve_root=EVOLVE_ROOT, threshold=0.02)

    assert [v["variant"] for v in result["variants"]] == ["v001"]
