"""Tests for src/fitness/runner.py.

Covers both compute_fitness (orchestration) and _aggregate (per-variant plugin
JSON -> fitness vector composition + contract enforcement).
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from src.fitness.runner import FitnessError, _aggregate, compute_fitness
from src.identifiers import CycleId, VariantId
from src.paths import CycleVariant, Layout, VariantLayout


def _layout(tmp_path: Path) -> Layout:
    evolve_root = tmp_path / "evolve"
    evolve_root.mkdir()
    return Layout(evolve_root=evolve_root, outputs_root=tmp_path / "out")


def _variant_layout(tmp_path: Path, i: int = 1) -> VariantLayout:
    root = tmp_path / "variants" / str(VariantId(i))
    (root / "impl").mkdir(parents=True)
    return VariantLayout(root=root)


def _stub_plugins(monkeypatch) -> None:
    def stub(dim, higher_is_better):
        def _compute(**_kwargs):
            return {"dim": dim, "values": [], "summary": 0.0, "higher_is_better": higher_is_better}
        return _compute

    monkeypatch.setattr("src.fitness.runner.hit_rate.compute", stub("hit-rate", True))
    monkeypatch.setattr("src.fitness.runner.per_op_time.compute", stub("per-op-time", False))
    monkeypatch.setattr("src.fitness.runner.metadata_size.compute", stub("metadata-size", False))


def test_writes_per_plugin_json_and_fitness_vector(tmp_path, monkeypatch):
    layout = _layout(tmp_path)
    variant_layout = _variant_layout(tmp_path)
    variant = CycleVariant(CycleId(1), variant_layout.variant_id)
    _stub_plugins(monkeypatch)

    compute_fitness(variant_layout, variant, layout=layout, repo_root=tmp_path)

    assert (variant_layout.fitness_dir / "hit-rate.json").is_file()
    assert (variant_layout.fitness_dir / "per-op-time.json").is_file()
    assert (variant_layout.fitness_dir / "metadata-size.json").is_file()
    fv = json.loads(variant_layout.fitness_vector_json.read_text())
    assert fv["variant"] == variant.label
    assert {d["dim"] for d in fv["dimensions"]} == {"hit-rate", "per-op-time", "metadata-size"}


def _write_plugin(plugin_dir, name, data):
    (plugin_dir / f"{name}.json").write_text(json.dumps(data))


def test_aggregate_composes_fitness_vector(tmp_path):
    _write_plugin(tmp_path, "hit-rate", {"dim": "hit-rate", "values": [], "summary": 0.65, "higher_is_better": True})
    _write_plugin(tmp_path, "per-op-time", {"dim": "per-op-time", "values": [], "summary": 95.0, "higher_is_better": False})
    _write_plugin(tmp_path, "metadata-size", {"dim": "metadata-size", "values": [], "summary": 4096.0, "higher_is_better": False})

    fv = _aggregate("v001", tmp_path)
    assert fv["variant"] == "v001"
    assert len(fv["dimensions"]) == 3
    assert {d["dim"] for d in fv["dimensions"]} == {"hit-rate", "per-op-time", "metadata-size"}


def test_aggregate_rejects_malformed_json(tmp_path):
    (tmp_path / "hit-rate.json").write_text("not valid json {{{")
    with pytest.raises(FitnessError, match="malformed JSON"):
        _aggregate("v001", tmp_path)


def test_aggregate_rejects_missing_required_fields(tmp_path):
    _write_plugin(tmp_path, "hit-rate", {"dim": "hit-rate", "summary": 0.5})
    with pytest.raises(FitnessError):
        _aggregate("v001", tmp_path)


def test_aggregate_empty_plugin_dir_errors(tmp_path):
    with pytest.raises(FitnessError, match="no plugin outputs"):
        _aggregate("v001", tmp_path)


def test_aggregate_values_propagated(tmp_path):
    _write_plugin(tmp_path, "hit-rate", {
        "dim": "hit-rate",
        "values": [{"cache": "dash", "size": 16, "hit_rate": 0.61}],
        "summary": 0.61,
        "higher_is_better": True,
    })
    fv = _aggregate("v001", tmp_path)
    [dim] = fv["dimensions"]
    assert dim["values"] == [{"cache": "dash", "size": 16, "hit_rate": 0.61}]


def test_aggregate_rejects_nan_summary(tmp_path):
    _write_plugin(tmp_path, "hit-rate", {
        "dim": "hit-rate",
        "values": [],
        "summary": float("nan"),
        "higher_is_better": True,
    })
    with pytest.raises(FitnessError, match=r"(?i)finite"):
        _aggregate("v001", tmp_path)
