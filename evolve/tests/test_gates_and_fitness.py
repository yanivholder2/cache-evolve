"""Tests for src/gates_and_fitness.py orchestration logic.

`run_gates` and the fitness plugins are mocked at the module boundary so the
orchestration (skip-on-failure, survivors, empty-cohort guard) is testable
without paying for cargo builds.
"""

from __future__ import annotations

from pathlib import Path

import pytest

from src.gate_check import GatesResult
from src.gates_and_fitness import run_gates_and_fitness
from src.identifiers import CycleId, VariantId
from src.paths import Layout, VariantLayout


def _layout(tmp_path: Path) -> Layout:
    evolve_root = tmp_path / "evolve"
    evolve_root.mkdir()
    return Layout(evolve_root=evolve_root, outputs_root=tmp_path / "out")


def _variant_layouts(tmp_path: Path, n: int) -> list[VariantLayout]:
    variants_dir = tmp_path / "variants"
    variants_dir.mkdir()
    layouts = []
    for i in range(1, n + 1):
        root = variants_dir / str(VariantId(i))
        (root / "impl").mkdir(parents=True)
        layouts.append(VariantLayout(root=root))
    return layouts


def _stub_fitness_plugins(monkeypatch) -> None:
    """Replace the cargo-shelling plugins with deterministic stubs."""

    def stub(dim, higher_is_better):
        def _compute(**_kwargs):
            return {"dim": dim, "values": [], "summary": 0.0, "higher_is_better": higher_is_better}
        return _compute

    monkeypatch.setattr("src.gates_and_fitness.hit_rate.compute", stub("hit-rate", True))
    monkeypatch.setattr("src.gates_and_fitness.per_op_time.compute", stub("per-op-time", False))
    monkeypatch.setattr("src.gates_and_fitness.metadata_size.compute", stub("metadata-size", False))


def test_failing_variant_is_skipped_and_survivors_returned(tmp_path, monkeypatch):
    layout = _layout(tmp_path)
    variants = _variant_layouts(tmp_path, 3)

    fails = {VariantId(2)}

    def fake_run_gates(_impl_dir, _cycle, variant_id, **_kwargs):
        return GatesResult(g1_passed=True, g2_passed=variant_id not in fails, g3_passed=variant_id not in fails)

    monkeypatch.setattr("src.gates_and_fitness.run_gates", fake_run_gates)
    _stub_fitness_plugins(monkeypatch)

    survivors = run_gates_and_fitness(
        variants, CycleId(1), layout=layout, repo_root=tmp_path,
    )

    assert [v.variant_id for v in survivors] == [VariantId(1), VariantId(3)]
    assert variants[0].fitness_vector_json.is_file()
    assert not variants[1].fitness_vector_json.exists()
    assert variants[2].fitness_vector_json.is_file()


def test_all_passing_returns_full_cohort(tmp_path, monkeypatch):
    layout = _layout(tmp_path)
    variants = _variant_layouts(tmp_path, 3)

    monkeypatch.setattr(
        "src.gates_and_fitness.run_gates",
        lambda *a, **kw: GatesResult(g1_passed=True, g2_passed=True, g3_passed=True),
    )
    _stub_fitness_plugins(monkeypatch)

    survivors = run_gates_and_fitness(
        variants, CycleId(1), layout=layout, repo_root=tmp_path,
    )

    assert survivors == variants


def test_zero_survivors_raises(tmp_path, monkeypatch):
    layout = _layout(tmp_path)
    variants = _variant_layouts(tmp_path, 2)

    monkeypatch.setattr(
        "src.gates_and_fitness.run_gates",
        lambda *a, **kw: GatesResult(g1_passed=False),
    )
    _stub_fitness_plugins(monkeypatch)

    with pytest.raises(RuntimeError, match="cycle 0001"):
        run_gates_and_fitness(variants, CycleId(1), layout=layout, repo_root=tmp_path)
