"""Tests for the cross-cycle winners computer (src/aggregate/cross_cycle_winners.py)."""

import json

from evolve._io import read_utf8, write_json
from evolve.aggregate.cross_cycle_winners import compute_cross_cycle_winners, write_cross_cycle_winners
from evolve.aggregate.selectors import ParetoSelector, WeightedSelector
from evolve.identifiers import CycleId, VariantId
from evolve.paths import Layout


def _pareto(*dims) -> ParetoSelector:
    return ParetoSelector(dimensions=tuple(dims))


def _vec(variant, *dims):
    # Each dim: (dim_name, summary, higher_is_better).
    return {
        "variant": variant,
        "dimensions": [
            {"dim": d, "values": [], "summary": s, "higher_is_better": h}
            for d, s, h in dims
        ],
    }


def _make_layout(tmp_path):
    return Layout(
        repo_root=tmp_path / "repo",
        outputs_root=tmp_path / "outputs",
        subject="dash",
    )


def _write_baseline(layout, fitness_vector):
    path = layout.sources.baseline.fitness_vector_json
    path.parent.mkdir(parents=True, exist_ok=True)
    write_json(path, fitness_vector)


def _write_variant(layout, cycle_id, variant_id, fitness_vector):
    path = layout.outputs.cycle(cycle_id).variant(variant_id).fitness_vector_json
    path.parent.mkdir(parents=True, exist_ok=True)
    write_json(path, fitness_vector)


def test_empty_returns_empty_structure(tmp_path):
    layout = _make_layout(tmp_path)
    data = compute_cross_cycle_winners(layout, _pareto("hit-rate"))
    assert data == {"winners": [], "scores": {}, "fitness_vectors": [], "best_per_dimension": {}}


def test_union_across_baseline_and_cycles(tmp_path):
    layout = _make_layout(tmp_path)
    # baseline: solid hit-rate, slow.
    _write_baseline(layout, _vec("baseline", ("hit-rate", 0.48, True), ("per-op-time", 30, False)))
    # cycle 1 v001: best hit-rate, slow.
    _write_variant(
        layout, CycleId(1), VariantId(1),
        _vec("v001", ("hit-rate", 0.60, True), ("per-op-time", 50, False)),
    )
    # cycle 2 v001: lower hit-rate, fastest.
    _write_variant(
        layout, CycleId(2), VariantId(1),
        _vec("v001", ("hit-rate", 0.40, True), ("per-op-time", 10, False)),
    )

    data = compute_cross_cycle_winners(layout, _pareto("hit-rate", "per-op-time"))

    labels = [fv["variant"] for fv in data["fitness_vectors"]]
    # Globally-unique ids; baseline first.
    assert labels == ["baseline", "cycle/0001/v001", "cycle/0002/v001"]
    assert len(set(labels)) == 3

    # baseline is dominated by cycle/0001 (better hit-rate) on time? No: baseline
    # time 30 beats cycle/0001's 50, so all three are mutually non-dominated here.
    assert set(data["winners"]) == {"baseline", "cycle/0001/v001", "cycle/0002/v001"}

    # higher-is-better dim: cycle/0001 holds best hit-rate.
    assert data["best_per_dimension"]["hit-rate"] == {
        "variant": "cycle/0001/v001",
        "summary": 0.60,
    }
    # lower-is-better dim: cycle/0002 holds best (lowest) per-op-time.
    assert data["best_per_dimension"]["per-op-time"] == {
        "variant": "cycle/0002/v001",
        "summary": 10,
    }


def test_dominated_baseline_excluded_from_front(tmp_path):
    layout = _make_layout(tmp_path)
    _write_baseline(layout, _vec("baseline", ("hit-rate", 0.30, True), ("per-op-time", 50, False)))
    # cycle 1 v001 strictly dominates baseline on both dims.
    _write_variant(
        layout, CycleId(1), VariantId(1),
        _vec("v001", ("hit-rate", 0.60, True), ("per-op-time", 20, False)),
    )

    data = compute_cross_cycle_winners(layout, _pareto("hit-rate", "per-op-time"))
    assert data["winners"] == ["cycle/0001/v001"]


def test_variant_without_fitness_vector_skipped(tmp_path):
    layout = _make_layout(tmp_path)
    _write_variant(
        layout, CycleId(1), VariantId(1),
        _vec("v001", ("hit-rate", 0.50, True)),
    )
    # v002 dir exists but has no fitness-vector.json.
    (layout.outputs.cycle(CycleId(1)).variant(VariantId(2)).root).mkdir(parents=True)

    data = compute_cross_cycle_winners(layout, _pareto("hit-rate"))
    labels = [fv["variant"] for fv in data["fitness_vectors"]]
    assert labels == ["cycle/0001/v001"]


def test_non_variant_dirs_ignored(tmp_path):
    layout = _make_layout(tmp_path)
    _write_variant(
        layout, CycleId(1), VariantId(1),
        _vec("v001", ("hit-rate", 0.50, True)),
    )
    # stray non-variant entries under variants/ and a non-cycle dir under cycles/.
    (layout.outputs.cycle(CycleId(1)).variants_dir / "hypotheses").mkdir(parents=True)
    (layout.outputs.cycles_dir / "not-a-cycle").mkdir(parents=True)

    data = compute_cross_cycle_winners(layout, _pareto("hit-rate"))
    labels = [fv["variant"] for fv in data["fitness_vectors"]]
    assert labels == ["cycle/0001/v001"]


def test_write_cross_cycle_winners_writes_file(tmp_path):
    layout = _make_layout(tmp_path)
    _write_baseline(layout, _vec("baseline", ("hit-rate", 0.48, True)))
    layout.outputs.root.mkdir(parents=True, exist_ok=True)

    returned = write_cross_cycle_winners(layout, _pareto("hit-rate"))

    on_disk = json.loads(read_utf8(layout.outputs.cross_cycle_winners_json))
    assert on_disk == returned
    assert on_disk["winners"] == ["baseline"]
    assert on_disk["scores"] == {}


def test_weighted_selector_top_k_and_scores(tmp_path):
    layout = _make_layout(tmp_path)
    _write_baseline(layout, _vec("baseline", ("hit-rate", 0.40, True)))
    _write_variant(layout, CycleId(1), VariantId(1), _vec("v001", ("hit-rate", 0.90, True)))
    _write_variant(layout, CycleId(1), VariantId(2), _vec("v002", ("hit-rate", 0.60, True)))

    selector = WeightedSelector(winners_count=2, weights={"hit-rate": 1.0})
    data = compute_cross_cycle_winners(layout, selector)

    # top-2 by hit-rate over {0.40, 0.90, 0.60}.
    assert data["winners"] == ["cycle/0001/v001", "cycle/0001/v002"]
    # scores cover every variant (min-max: v001=1.0, baseline=0.0).
    assert set(data["scores"]) == {"baseline", "cycle/0001/v001", "cycle/0001/v002"}
    assert data["scores"]["cycle/0001/v001"] == 1.0
    assert data["scores"]["baseline"] == 0.0
