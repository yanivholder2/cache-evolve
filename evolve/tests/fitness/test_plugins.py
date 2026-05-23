"""Tests for fitness plugin output shape."""

from pathlib import Path

from src.fitness.plugins import hit_rate, metadata_size, per_op_time

REPO_ROOT = Path(__file__).parent.parent.parent.parent
EVOLVE_ROOT = Path(__file__).parent.parent.parent


def _assert_shape(data, expected_dim):
    assert data["dim"] == expected_dim
    assert isinstance(data["values"], list)
    assert isinstance(data["summary"], (int, float))
    assert isinstance(data["higher_is_better"], bool)


def test_hit_rate_plugin_shape():
    data = hit_rate.compute(repo_root=REPO_ROOT)
    _assert_shape(data, "hit-rate")
    assert data["higher_is_better"] is True


def test_per_op_time_plugin_shape():
    data = per_op_time.compute(evolve_root=EVOLVE_ROOT)
    _assert_shape(data, "per-op-time")
    assert data["higher_is_better"] is False
    if data["values"]:
        assert "p50_ns" in data["values"][0]
        assert "p99_ns" in data["values"][0]


def test_metadata_size_plugin_shape():
    data = metadata_size.compute(repo_root=REPO_ROOT)
    _assert_shape(data, "metadata-size")
    assert data["higher_is_better"] is False
    assert data["summary"] > 0
