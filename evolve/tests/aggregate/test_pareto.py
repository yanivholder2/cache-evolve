"""Tests for the Pareto front computer (src/aggregate/pareto.py)."""

import pytest

from src.aggregate.pareto import compute_pareto_front


def _vec(variant, **dims):
    # Each kwarg: dim_name=(summary, higher_is_better).
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
    # v001 strictly dominates v002 on every dim.
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
    vecs = [
        _vec("v001", hit_rate=(0.8, True), per_op_time=(120.0, False)),
        _vec("v002", hit_rate=(0.5, True), per_op_time=(60.0, False)),
    ]
    front = compute_pareto_front(vecs)
    assert set(front) == {"v001", "v002"}


def test_dimension_tie_both_on_front():
    vecs = [
        _vec("v001", hit_rate=(0.65, True)),
        _vec("v002", hit_rate=(0.65, True)),
    ]
    front = compute_pareto_front(vecs)
    assert set(front) == {"v001", "v002"}


def test_mismatched_dimensions_raises():
    vecs = [
        _vec("v001", hit_rate=(0.7, True)),
        _vec("v002", per_op_time=(80.0, False)),
    ]
    with pytest.raises(ValueError, match="uniform dimensions"):
        compute_pareto_front(vecs)


def test_nan_summary_raises():
    vecs = [_vec("v001", hit_rate=(float("nan"), True))]
    with pytest.raises(ValueError, match="non-finite"):
        compute_pareto_front(vecs)


def test_inf_summary_raises():
    vecs = [_vec("v001", hit_rate=(float("inf"), True))]
    with pytest.raises(ValueError, match="non-finite"):
        compute_pareto_front(vecs)


def test_three_way_partial_dominance():
    # v002 dominates v003 on both dims; v001 and v002 are mutually
    # non-dominated (v001 wins hit-rate, v002 wins time).
    vecs = [
        _vec("v001", hit_rate=(0.9, True), per_op_time=(150.0, False)),
        _vec("v002", hit_rate=(0.6, True), per_op_time=(90.0, False)),
        _vec("v003", hit_rate=(0.3, True), per_op_time=(200.0, False)),
    ]
    front = compute_pareto_front(vecs)
    assert set(front) == {"v001", "v002"}
