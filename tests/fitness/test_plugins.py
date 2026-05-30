"""Tests for fitness plugin output shape."""

from pathlib import Path

import pytest

from evolve.fitness.caffeine_simulator import CaffeineResult, CaffeineResults
from evolve.fitness.plugins import hit_rate, metadata_size, per_op_time
from evolve.identifiers import CycleId, VariantId
from evolve.paths import CargoWorkspace, CycleVariant, Layout

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


def _per_op_run(p50_get, p99_get, p50_put, p99_put, summary):
    """A single per-op-time run record matching the plugin contract."""
    return {
        "dim": "per-op-time",
        "values": [
            {"op": "get", "p50_ns": p50_get, "p99_ns": p99_get},
            {"op": "put", "p50_ns": p50_put, "p99_ns": p99_put},
        ],
        "summary": summary,
        "higher_is_better": False,
    }


def test_per_op_time_aggregates_median_and_spread(monkeypatch):
    layout = Layout(repo_root=REPO_ROOT, outputs_root=REPO_ROOT / "outputs", subject=SUBJECT)
    monkeypatch.setattr(per_op_time, "cargo_build", lambda *_a, **_kw: None)
    monkeypatch.setattr(per_op_time, "_resolve_target_dir", lambda *_a, **_kw: Path("/tmp"))

    runs = [
        _per_op_run(10, 20, 100, 200, 5.0),
        _per_op_run(14, 24, 110, 210, 9.0),
        _per_op_run(12, 22, 105, 205, 7.0),
    ]
    scripted = iter(runs)
    monkeypatch.setattr(per_op_time, "_run_once", lambda *_a, **_kw: next(scripted))

    data = per_op_time.compute(layout=layout, repeats=3)
    _assert_shape(data, "per-op-time")
    assert data["higher_is_better"] is False
    assert data["summary"] == pytest.approx(7.0)  # median of 5, 9, 7

    get = next(v for v in data["values"] if v["op"] == "get")
    assert get["p50_ns"] == pytest.approx(12)  # median of 10, 14, 12
    assert get["p99_ns"] == pytest.approx(22)
    assert get["p50_ns_spread"] == [10, 14]
    assert get["p99_ns_spread"] == [20, 24]

    put = next(v for v in data["values"] if v["op"] == "put")
    assert put["p50_ns"] == pytest.approx(105)
    assert put["p50_ns_spread"] == [100, 110]


def test_per_op_time_single_repeat(monkeypatch):
    layout = Layout(repo_root=REPO_ROOT, outputs_root=REPO_ROOT / "outputs", subject=SUBJECT)
    monkeypatch.setattr(per_op_time, "cargo_build", lambda *_a, **_kw: None)
    monkeypatch.setattr(per_op_time, "_resolve_target_dir", lambda *_a, **_kw: Path("/tmp"))
    monkeypatch.setattr(per_op_time, "_run_once", lambda *_a, **_kw: _per_op_run(10, 20, 100, 200, 5.0))

    data = per_op_time.compute(layout=layout, repeats=1)
    get = next(v for v in data["values"] if v["op"] == "get")
    assert get["p50_ns"] == 10
    assert get["p50_ns_spread"] == [10, 10]
    assert get["p99_ns_spread"] == [20, 20]
    assert data["summary"] == pytest.approx(5.0)


def test_metadata_size_builds_against_overlay(monkeypatch, tmp_path):
    layout = Layout(repo_root=REPO_ROOT, outputs_root=REPO_ROOT / "outputs", subject=SUBJECT)
    variant = CycleVariant(CycleId(1), VariantId(1))

    workspace_root = tmp_path / "variant-workspace"
    workspace_root.mkdir()
    workspace = CargoWorkspace(root=workspace_root, cdylib_name="stub")
    seen = {}

    def fake_prepare(_variant, **_kw):
        return workspace

    def fake_cargo_build(cwd, args, **_kw):
        seen["cwd"] = cwd
        seen["args"] = args

    class _Result:
        stdout = '{"dim": "metadata-size", "values": [], "summary": 48.0, "higher_is_better": false}'

    monkeypatch.setattr(metadata_size, "prepare_variant_workspace", fake_prepare)
    monkeypatch.setattr(metadata_size, "cargo_build", fake_cargo_build)
    monkeypatch.setattr(metadata_size, "run_binary", lambda *_a, **_kw: _Result())

    data = metadata_size.compute(
        variant=variant, layout=layout, repo_root=REPO_ROOT, binary_name="evolve-metadata-size"
    )
    _assert_shape(data, "metadata-size")
    assert data["higher_is_better"] is False
    assert data["summary"] == pytest.approx(48.0)
    # Built inside the variant's overlaid workspace, selecting the bin by name.
    assert seen["cwd"] == workspace_root
    assert seen["args"] == ["--bin", "evolve-metadata-size"]
    # Workspace cleaned up on exit.
    assert not workspace_root.exists()
