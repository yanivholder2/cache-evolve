"""Tests for src/fitness/runner.py.

Covers both compute_fitness (orchestration) and _aggregate (per-variant plugin
JSON -> fitness vector composition + contract enforcement).
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from evolve._io import read_utf8, write_utf8
from evolve.config import Configs, FrameworkConfig, SimulatorConfig
from evolve.fitness.runner import FitnessError, _aggregate, compute_fitness
from evolve.identifiers import CycleId, VariantId
from evolve.paths import CycleVariant, Layout

from _support import configured_subject


REPO_ROOT = Path(__file__).resolve().parents[2]
SUBJECT = configured_subject(REPO_ROOT)


_TEST_CFG = Configs(
    framework=FrameworkConfig(
        cycles=1, cohort_size=1, ai_agent_tool="claude-sdk",
        outputs_root=Path("/tmp"), log_level="DEBUG", digest_cycles=3,
        max_workers=0, implementation_attempts=3, hypothesis_attempts=3,
        authoring_scope="extension-points", subject=SUBJECT,
    ),
    simulator=SimulatorConfig(timeout_s=600),
)


def _stage(tmp_path: Path) -> tuple[Layout, CycleVariant]:
    """Build a Layout whose sources point at the real repo (so the subject
    manifest loads) and whose outputs live under tmp_path."""

    layout = Layout(repo_root=REPO_ROOT, outputs_root=tmp_path / "out", subject=SUBJECT)
    variant = CycleVariant(CycleId(1), VariantId(1))
    layout.outputs.cycle(variant.cycle_id).variant(variant.variant_id).overlay_dir.mkdir(parents=True)
    return layout, variant


def _stub_plugins(monkeypatch) -> None:
    def stub(dim, higher_is_better):
        def _compute(**_kwargs):
            return {"dim": dim, "values": [], "summary": 0.0, "higher_is_better": higher_is_better}
        return _compute

    # Stub a subject declaring every optional plugin so the orchestration path
    # (one JSON per scheduled plugin + aggregation) is covered regardless of any
    # real subject manifest.
    class _NamedBinary:
        binary_name = "stub"

    class _Subject:
        name = "stub"
        bench = _NamedBinary()
        metadata = _NamedBinary()

    from evolve.fitness.caffeine_simulator import CaffeineResults
    monkeypatch.setattr("evolve.fitness.runner.load_subject", lambda *_a, **_kw: _Subject())
    monkeypatch.setattr(
        "evolve.fitness.runner.simulate_variant",
        lambda *_a, **_kw: CaffeineResults(results=[]),
    )
    monkeypatch.setattr("evolve.fitness.runner.hit_rate.compute", stub("hit-rate", True))
    monkeypatch.setattr("evolve.fitness.runner.per_op_time.compute", stub("per-op-time", False))
    monkeypatch.setattr("evolve.fitness.runner.metadata_size.compute", stub("metadata-size", False))


def test_writes_per_plugin_json_and_fitness_vector(tmp_path, monkeypatch):
    layout, variant = _stage(tmp_path)
    _stub_plugins(monkeypatch)

    compute_fitness(variant, layout=layout, repo_root=tmp_path, cfg=_TEST_CFG)

    variant_dir = variant.resolve(layout)
    assert (variant_dir.fitness_dir / "hit-rate.json").is_file()
    assert (variant_dir.fitness_dir / "per-op-time.json").is_file()
    assert (variant_dir.fitness_dir / "metadata-size.json").is_file()
    fv = json.loads(read_utf8(variant_dir.fitness_vector_json))
    assert fv["variant"] == variant.label
    assert {d["dim"] for d in fv["dimensions"]} == {"hit-rate", "per-op-time", "metadata-size"}


def _write_plugin(plugin_dir, name, data):
    write_utf8(plugin_dir / f"{name}.json", json.dumps(data))


def test_aggregate_composes_fitness_vector(tmp_path):
    _write_plugin(tmp_path, "hit-rate", {"dim": "hit-rate", "values": [], "summary": 0.65, "higher_is_better": True})
    _write_plugin(tmp_path, "per-op-time", {"dim": "per-op-time", "values": [], "summary": 95.0, "higher_is_better": False})
    _write_plugin(tmp_path, "metadata-size", {"dim": "metadata-size", "values": [], "summary": 4096.0, "higher_is_better": False})

    fv = _aggregate("v001", tmp_path)
    assert fv["variant"] == "v001"
    assert len(fv["dimensions"]) == 3
    assert {d["dim"] for d in fv["dimensions"]} == {"hit-rate", "per-op-time", "metadata-size"}


def test_aggregate_rejects_malformed_json(tmp_path):
    write_utf8(tmp_path / "hit-rate.json", "not valid json {{{")
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
        "values": [{"cache": "stub", "size": 16, "hit_rate": 0.61}],
        "summary": 0.61,
        "higher_is_better": True,
    })
    fv = _aggregate("v001", tmp_path)
    [dim] = fv["dimensions"]
    assert dim["values"] == [{"cache": "stub", "size": 16, "hit_rate": 0.61}]


def test_aggregate_rejects_nan_summary(tmp_path):
    _write_plugin(tmp_path, "hit-rate", {
        "dim": "hit-rate",
        "values": [],
        "summary": float("nan"),
        "higher_is_better": True,
    })
    with pytest.raises(FitnessError, match=r"(?i)finite"):
        _aggregate("v001", tmp_path)
