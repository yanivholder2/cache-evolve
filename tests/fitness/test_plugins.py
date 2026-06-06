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


def test_per_op_time_builds_against_overlay(monkeypatch, tmp_path):
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
        stdout = (
            '{"dim": "per-op-time", '
            '"values": [{"op": "get", "p50_ns": 10, "p99_ns": 20}, '
            '{"op": "put", "p50_ns": 100, "p99_ns": 200}], '
            '"summary": 5.0, "higher_is_better": false}'
        )

    monkeypatch.setattr(per_op_time, "prepare_variant_workspace", fake_prepare)
    monkeypatch.setattr(per_op_time, "cargo_build", fake_cargo_build)
    monkeypatch.setattr(per_op_time, "run_binary", lambda *_a, **_kw: _Result())

    data = per_op_time.compute(
        variant=variant, layout=layout, repo_root=REPO_ROOT, binary_name="evolve-per-op-time"
    )
    _assert_shape(data, "per-op-time")
    assert data["higher_is_better"] is False
    get = next(v for v in data["values"] if v["op"] == "get")
    assert get["p50_ns"] == 10
    assert get["p99_ns"] == 20
    # Built --release inside the variant's overlaid workspace, selecting the bin by name.
    assert seen["cwd"] == workspace_root
    assert seen["args"] == ["--release", "--bin", "evolve-per-op-time"]
    # Workspace cleaned up on exit.
    assert not workspace_root.exists()


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
