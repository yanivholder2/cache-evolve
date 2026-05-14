"""Tests for the Pareto front computer (scripts/pareto.py)."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent / "scripts"))
from pareto import compute_pareto_front  # noqa: E402


def _vec(variant, **dims):
    """Build a FitnessVector dict. Each kwarg: dim_name=(summary, higher_is_better)."""
    return {
        "variant": variant,
        "dimensions": [
            {"dim": k, "summary": v[0], "higher_is_better": v[1]} for k, v in dims.items()
        ],
    }


def test_empty_returns_empty():
    assert compute_pareto_front([]) == []


def test_single_variant_is_on_front():
    vecs = [_vec("v001", hit_rate=(0.5, True))]
    assert compute_pareto_front(vecs) == ["v001"]


def test_dominated_variant_excluded():
    # v001 dominates v002: better hit-rate AND lower (better) per-op-time
    vecs = [
        _vec("v001", hit_rate=(0.7, True), per_op_time=(80.0, False)),
        _vec("v002", hit_rate=(0.5, True), per_op_time=(120.0, False)),
    ]
    assert compute_pareto_front(vecs) == ["v001"]


def test_all_dominated_only_best_remains():
    vecs = [
        _vec("v001", hit_rate=(0.9, True)),
        _vec("v002", hit_rate=(0.6, True)),
        _vec("v003", hit_rate=(0.3, True)),
    ]
    assert compute_pareto_front(vecs) == ["v001"]


def test_mutually_nondominated_all_on_front():
    # v001: great hit-rate, slow; v002: poor hit-rate, fast — neither dominates
    vecs = [
        _vec("v001", hit_rate=(0.8, True), per_op_time=(120.0, False)),
        _vec("v002", hit_rate=(0.5, True), per_op_time=(60.0, False)),
    ]
    front = compute_pareto_front(vecs)
    assert set(front) == {"v001", "v002"}


def test_dimension_tie_both_on_front():
    # Identical scores — neither strictly dominates the other
    vecs = [
        _vec("v001", hit_rate=(0.65, True)),
        _vec("v002", hit_rate=(0.65, True)),
    ]
    front = compute_pareto_front(vecs)
    assert set(front) == {"v001", "v002"}


def test_three_way_partial_dominance():
    # v001: best hit-rate, worst time
    # v002: middle both
    # v003: worst hit-rate, best time
    # v001 dominates v002? v001 hit_rate>v002 but v001 time > v002 (worse) → NOT dominated
    # v002 hit_rate > v003 AND v002 time < v003 → v002 dominates v003
    vecs = [
        _vec("v001", hit_rate=(0.9, True), per_op_time=(150.0, False)),
        _vec("v002", hit_rate=(0.6, True), per_op_time=(90.0, False)),
        _vec("v003", hit_rate=(0.3, True), per_op_time=(200.0, False)),
    ]
    front = compute_pareto_front(vecs)
    # v003 is dominated by v002; v001 and v002 are mutually non-dominated
    assert set(front) == {"v001", "v002"}
