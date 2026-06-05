"""Unit tests for src/evolve/cli.py helpers."""

from __future__ import annotations

import types
from pathlib import Path

import pytest

from evolve._io import write_json
from evolve.cli import _require_scope_supported, _snapshot_run_inputs, _validate_active_dimensions
from evolve.config import (
    AuthoringScope,
    Configs,
    FitnessConfig,
    FrameworkConfig,
    SelectionConfig,
    SimulatorConfig,
)
from evolve.paths import Layout


def _stub_subject(extension_points: tuple = ()) -> types.SimpleNamespace:
    return types.SimpleNamespace(name="lru_cache", extension_points=extension_points)


def _cfg(dimensions: tuple[str, ...]) -> Configs:
    return Configs(
        framework=FrameworkConfig(
            cycles=1, cohort_size=1, ai_agent_tool="claude-sdk",
            outputs_root=Path("/tmp"), log_level="DEBUG", digest_cycles=3,
            max_workers=0, implementation_attempts=3, hypothesis_attempts=3,
            authoring_scope="full-autonomy", subject="lru_cache",
        ),
        simulator=SimulatorConfig(timeout_s=600),
        fitness=FitnessConfig(dimensions=dimensions),
        selection=SelectionConfig(method="pareto", winners_count=3, weights=None),
    )


def _dim_subject(*, bench=None, metadata=None) -> types.SimpleNamespace:
    return types.SimpleNamespace(name="lru_cache", bench=bench, metadata=metadata)


def _layout_with_baseline(tmp_path: Path, baseline_dims: list[str]) -> Layout:
    layout = Layout(repo_root=tmp_path / "repo", outputs_root=tmp_path / "out", subject="lru_cache")
    layout.sources.baseline.fitness_vector_json.parent.mkdir(parents=True, exist_ok=True)
    write_json(
        layout.sources.baseline.fitness_vector_json,
        {"variant": "baseline", "dimensions": [{"dim": d, "summary": 0.5, "higher_is_better": True} for d in baseline_dims]},
    )
    return layout


def test_validate_dimensions_happy_path(tmp_path):
    layout = _layout_with_baseline(tmp_path, ["hit-rate"])
    _validate_active_dimensions(_cfg(("hit-rate",)), _dim_subject(), layout)


def test_validate_dimensions_unknown_dimension_exits(tmp_path):
    layout = _layout_with_baseline(tmp_path, ["hit-rate"])
    with pytest.raises(SystemExit):
        _validate_active_dimensions(_cfg(("made-up",)), _dim_subject(), layout)


def test_validate_dimensions_missing_capability_exits(tmp_path):
    layout = _layout_with_baseline(tmp_path, ["hit-rate", "per-op-time"])
    # subject declares no [bench], so per-op-time cannot be activated.
    with pytest.raises(SystemExit):
        _validate_active_dimensions(_cfg(("per-op-time",)), _dim_subject(bench=None), layout)


def test_validate_dimensions_not_in_baseline_exits(tmp_path):
    layout = _layout_with_baseline(tmp_path, ["hit-rate"])
    # metadata-size is subject-capable but absent from the baseline anchor.
    with pytest.raises(SystemExit):
        _validate_active_dimensions(
            _cfg(("metadata-size",)), _dim_subject(metadata=object()), layout
        )


def test_extension_points_scope_rejects_subject_without_extension_points():
    with pytest.raises(SystemExit):
        _require_scope_supported(_stub_subject(()), AuthoringScope.EXTENSION_POINTS)


def test_extension_points_scope_allows_subject_with_extension_points():
    _require_scope_supported(
        _stub_subject(("ep",)), AuthoringScope.EXTENSION_POINTS
    )


def test_full_autonomy_scope_allows_subject_without_extension_points():
    _require_scope_supported(_stub_subject(()), AuthoringScope.FULL_AUTONOMY)


def test_snapshot_run_inputs_copies_config_and_manifest(tmp_path):
    layout = Layout(repo_root=tmp_path / "repo", outputs_root=tmp_path / "out", subject="lru_cache")
    layout.sources.config_toml.parent.mkdir(parents=True, exist_ok=True)
    layout.sources.config_toml.write_text("cycles = 7\n")
    layout.sources.subject_manifest.parent.mkdir(parents=True, exist_ok=True)
    layout.sources.subject_manifest.write_text("[caffeine]\n")

    _snapshot_run_inputs(layout)

    assert layout.outputs.config_toml.read_text() == "cycles = 7\n"
    assert layout.outputs.manifest_toml.read_text() == "[caffeine]\n"
