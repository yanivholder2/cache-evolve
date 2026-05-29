"""Tests for fitness plugin output shape."""

from pathlib import Path

import pytest

from evolve.fitness.caffeine_simulator import CaffeineResult, CaffeineResults
from evolve.fitness.plugins import hit_rate, metadata_size, per_op_time
from evolve.paths import Layout
from evolve.subject import load_subject

from _support import configured_subject

REPO_ROOT = Path(__file__).parent.parent.parent
SUBJECT = configured_subject(REPO_ROOT)


def _assert_shape(data, expected_dim):
    assert data["dim"] == expected_dim
    assert isinstance(data["values"], list)
    assert isinstance(data["summary"], (int, float))
    assert isinstance(data["higher_is_better"], bool)


def _fake_result(hit_rate_value: float) -> CaffeineResult:
    return CaffeineResult(
        trace="lirs:multi1.trace.gz",
        size=512,
        policy="classiclru",
        hit_rate=hit_rate_value,
        miss_rate=1.0 - hit_rate_value,
        hits=int(1000 * hit_rate_value),
        misses=int(1000 * (1.0 - hit_rate_value)),
        requests=1000,
        evictions=10,
    )


def test_hit_rate_plugin_shape():
    results = CaffeineResults(results=[_fake_result(0.42)])
    data = hit_rate.compute(results=results)
    _assert_shape(data, "hit-rate")
    assert data["higher_is_better"] is True
    assert data["summary"] == pytest.approx(0.42)
    assert data["values"] == [{"trace": "lirs:multi1.trace.gz", "size": 512, "hit_rate": 0.42}]


def test_hit_rate_plugin_averages_multiple_points():
    results = CaffeineResults(results=[_fake_result(0.3), _fake_result(0.5)])
    data = hit_rate.compute(results=results)
    assert data["summary"] == pytest.approx(0.4)
    assert len(data["values"]) == 2


def test_hit_rate_plugin_rejects_empty_results():
    with pytest.raises(RuntimeError):
        hit_rate.compute(results=CaffeineResults(results=[]))


def test_per_op_time_plugin_shape():
    layout = Layout(repo_root=REPO_ROOT, outputs_root=REPO_ROOT / "outputs", subject=SUBJECT)
    data = per_op_time.compute(layout=layout)
    _assert_shape(data, "per-op-time")
    assert data["higher_is_better"] is False
    if data["values"]:
        assert "p50_ns" in data["values"][0]
        assert "p99_ns" in data["values"][0]


def test_metadata_size_plugin_shape():
    layout = Layout(repo_root=REPO_ROOT, outputs_root=REPO_ROOT / "outputs", subject=SUBJECT)
    subject = load_subject(layout.sources.subject_dir)
    data = metadata_size.compute(
        repo_root=REPO_ROOT, package=subject.name, binary_name=subject.metadata.binary_name
    )
    _assert_shape(data, "metadata-size")
    assert data["higher_is_better"] is False
    assert data["summary"] > 0
