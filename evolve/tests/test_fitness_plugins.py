"""Verify each fitness plugin is invokable standalone and emits the documented JSON shape.

Shape: { "dim": str, "values": list, "summary": float, "higher_is_better": bool }
"""

import json
import os
import subprocess
import sys
from pathlib import Path

SCRIPTS_DIR = Path(__file__).parent.parent / "scripts"
REPO_ROOT = Path(__file__).parent.parent.parent
EVOLVE_ROOT = Path(__file__).parent.parent


def _run_plugin(name, extra_args=None, env_overrides=None):
    script = SCRIPTS_DIR / "fitness" / f"{name}.py"
    env = {**os.environ, **(env_overrides or {})}
    cmd = [sys.executable, str(script)] + (extra_args or [])
    return subprocess.run(cmd, capture_output=True, text=True, env=env)


def _assert_shape(data, expected_dim):
    assert data["dim"] == expected_dim
    assert isinstance(data["values"], list)
    assert isinstance(data["summary"], (int, float))
    assert isinstance(data["higher_is_better"], bool)


def test_hit_rate_plugin_shape(tmp_path):
    result = _run_plugin(
        "hit-rate",
        env_overrides={"REPO_ROOT": str(REPO_ROOT), "EVOLVE_ROOT": str(EVOLVE_ROOT)},
    )
    assert result.returncode == 0, result.stderr
    data = json.loads(result.stdout)
    _assert_shape(data, "hit-rate")
    assert data["higher_is_better"] is True


def test_per_op_time_plugin_shape(tmp_path):
    result = _run_plugin(
        "per-op-time",
        env_overrides={"REPO_ROOT": str(REPO_ROOT), "EVOLVE_ROOT": str(EVOLVE_ROOT)},
    )
    assert result.returncode == 0, result.stderr
    data = json.loads(result.stdout)
    _assert_shape(data, "per-op-time")
    assert data["higher_is_better"] is False
    # values should have p50_ns and p99_ns entries
    if data["values"]:
        assert "p50_ns" in data["values"][0]
        assert "p99_ns" in data["values"][0]


def test_metadata_size_plugin_shape(tmp_path):
    result = _run_plugin(
        "metadata-size",
        env_overrides={"REPO_ROOT": str(REPO_ROOT), "EVOLVE_ROOT": str(EVOLVE_ROOT)},
    )
    assert result.returncode == 0, result.stderr
    data = json.loads(result.stdout)
    _assert_shape(data, "metadata-size")
    assert data["higher_is_better"] is False
    # summary should be a positive number of bytes
    assert data["summary"] > 0
