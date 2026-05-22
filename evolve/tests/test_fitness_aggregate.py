"""Tests for src/fitness_aggregate.aggregate."""

import json

import pytest

from src.fitness_aggregate import FitnessAggregateError, aggregate


def _write_plugin(plugin_dir, name, data):
    (plugin_dir / f"{name}.json").write_text(json.dumps(data))


def test_composes_fitness_vector(tmp_path):
    _write_plugin(tmp_path, "hit-rate", {"dim": "hit-rate", "values": [], "summary": 0.65, "higher_is_better": True})
    _write_plugin(tmp_path, "per-op-time", {"dim": "per-op-time", "values": [], "summary": 95.0, "higher_is_better": False})
    _write_plugin(tmp_path, "metadata-size", {"dim": "metadata-size", "values": [], "summary": 4096.0, "higher_is_better": False})

    fv = aggregate("v001", tmp_path)
    assert fv["variant"] == "v001"
    assert len(fv["dimensions"]) == 3
    dims = {d["dim"] for d in fv["dimensions"]}
    assert dims == {"hit-rate", "per-op-time", "metadata-size"}


def test_rejects_malformed_json(tmp_path):
    (tmp_path / "hit-rate.json").write_text("not valid json {{{")
    with pytest.raises(FitnessAggregateError, match="malformed JSON"):
        aggregate("v001", tmp_path)


def test_rejects_missing_required_fields(tmp_path):
    # Missing 'higher_is_better'
    _write_plugin(tmp_path, "hit-rate", {"dim": "hit-rate", "summary": 0.5})
    with pytest.raises(FitnessAggregateError):
        aggregate("v001", tmp_path)


def test_empty_plugin_dir_errors(tmp_path):
    with pytest.raises(FitnessAggregateError, match="no plugin outputs"):
        aggregate("v001", tmp_path)


def test_values_propagated(tmp_path):
    _write_plugin(tmp_path, "hit-rate", {
        "dim": "hit-rate",
        "values": [{"cache": "dash", "size": 16, "hit_rate": 0.61}],
        "summary": 0.61,
        "higher_is_better": True,
    })
    fv = aggregate("v001", tmp_path)
    [dim] = fv["dimensions"]
    assert dim["values"] == [{"cache": "dash", "size": 16, "hit_rate": 0.61}]


def test_rejects_nan_summary(tmp_path):
    _write_plugin(tmp_path, "hit-rate", {
        "dim": "hit-rate",
        "values": [],
        "summary": float("nan"),
        "higher_is_better": True,
    })
    with pytest.raises(FitnessAggregateError, match=r"(?i)finite"):
        aggregate("v001", tmp_path)
