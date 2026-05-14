"""Tests for scripts/fitness-aggregate.py."""

import json
import subprocess
import sys
from pathlib import Path

SCRIPTS_DIR = Path(__file__).parent.parent / "scripts"
AGGREGATE = SCRIPTS_DIR / "fitness-aggregate.py"


def _run(variant, plugin_dir, env=None):
    import os
    e = {**os.environ, **(env or {})}
    return subprocess.run(
        [sys.executable, str(AGGREGATE), variant, str(plugin_dir)],
        capture_output=True,
        text=True,
        env=e,
    )


def _write_plugin(plugin_dir, name, data):
    (plugin_dir / f"{name}.json").write_text(json.dumps(data))


def test_composes_fitness_vector(tmp_path):
    _write_plugin(tmp_path, "hit-rate", {"dim": "hit-rate", "values": [], "summary": 0.65, "higher_is_better": True})
    _write_plugin(tmp_path, "per-op-time", {"dim": "per-op-time", "values": [], "summary": 95.0, "higher_is_better": False})
    _write_plugin(tmp_path, "metadata-size", {"dim": "metadata-size", "values": [], "summary": 4096.0, "higher_is_better": False})

    result = _run("v001", tmp_path)
    assert result.returncode == 0, result.stderr

    fv = json.loads(result.stdout)
    assert fv["variant"] == "v001"
    assert "dimensions" in fv
    assert len(fv["dimensions"]) == 3
    dims = {d["dim"] for d in fv["dimensions"]}
    assert dims == {"hit-rate", "per-op-time", "metadata-size"}


def test_includes_worker_id(tmp_path):
    _write_plugin(tmp_path, "hit-rate", {"dim": "hit-rate", "values": [], "summary": 0.5, "higher_is_better": True})
    result = _run("v001", tmp_path, env={"WORKER_ID": "worker-42"})
    assert result.returncode == 0
    fv = json.loads(result.stdout)
    assert fv["worker_id"] == "worker-42"


def test_rejects_malformed_json(tmp_path):
    (tmp_path / "hit-rate.json").write_text("not valid json {{{")
    result = _run("v001", tmp_path)
    assert result.returncode != 0
    assert result.stderr.strip()  # must print a clear error


def test_rejects_missing_required_fields(tmp_path):
    # Missing 'higher_is_better'
    _write_plugin(tmp_path, "hit-rate", {"dim": "hit-rate", "summary": 0.5})
    result = _run("v001", tmp_path)
    assert result.returncode != 0
    assert result.stderr.strip()


def test_empty_plugin_dir_produces_empty_dimensions(tmp_path):
    result = _run("v001", tmp_path)
    assert result.returncode == 0
    fv = json.loads(result.stdout)
    assert fv["dimensions"] == []
