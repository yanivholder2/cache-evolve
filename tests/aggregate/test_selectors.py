"""Tests for the selection seam: registry, build_selector, projection, and the
pareto/weighted selectors' select() behavior."""

import pytest

from evolve.aggregate.selectors import (
    SELECTOR_REGISTRY,
    ParetoSelector,
    SelectorConfigError,
    WeightedSelector,
    build_selector,
)
from evolve.aggregate.selectors.base import project


def _vec(variant, **dims):
    # Each kwarg: dim_name=(summary, higher_is_better).
    return {
        "variant": variant,
        "dimensions": [
            {"dim": k, "summary": v[0], "higher_is_better": v[1]} for k, v in dims.items()
        ],
    }


# --- registry / build_selector ------------------------------------------------


def test_registry_holds_both_builtins():
    assert set(SELECTOR_REGISTRY) == {"pareto", "weighted"}


def test_build_selector_unknown_method_raises():
    with pytest.raises(SelectorConfigError, match="must be one of"):
        build_selector(method="sideways", winners_count=1, dimensions=("hit-rate",), weights=None)


def test_build_pareto_rejects_weights():
    with pytest.raises(SelectorConfigError, match="must be absent"):
        build_selector(
            method="pareto", winners_count=1, dimensions=("hit-rate",), weights={"hit-rate": 1.0}
        )


def test_build_weighted_requires_weights():
    with pytest.raises(SelectorConfigError, match="required"):
        build_selector(method="weighted", winners_count=1, dimensions=("hit-rate",), weights=None)


def test_build_weighted_weight_keys_must_match_dimensions():
    with pytest.raises(SelectorConfigError, match="must exactly match"):
        build_selector(
            method="weighted", winners_count=1, dimensions=("hit-rate",),
            weights={"per-op-time": 1.0},
        )


def test_build_returns_correct_types():
    assert isinstance(
        build_selector(method="pareto", winners_count=1, dimensions=("hit-rate",), weights=None),
        ParetoSelector,
    )
    assert isinstance(
        build_selector(
            method="weighted", winners_count=1, dimensions=("hit-rate",), weights={"hit-rate": 1.0}
        ),
        WeightedSelector,
    )


# --- projection ---------------------------------------------------------------


def test_project_restricts_to_active_dimensions_in_order():
    vecs = [_vec("v001", b=(2.0, True), a=(1.0, True), c=(3.0, True))]
    projected = project(vecs, ("a", "b"))
    assert [d["dim"] for d in projected[0]["dimensions"]] == ["a", "b"]


def test_project_missing_active_dimension_raises():
    vecs = [_vec("v001", a=(1.0, True))]
    with pytest.raises(ValueError, match="missing active dimensions"):
        project(vecs, ("a", "b"))


# --- pareto selector ----------------------------------------------------------


def test_pareto_select_returns_front_and_empty_scores():
    selector = ParetoSelector(dimensions=("hit-rate",))
    result = selector.select([
        _vec("v001", **{"hit-rate": (0.9, True)}),
        _vec("v002", **{"hit-rate": (0.4, True)}),
    ])
    assert result.winners == ["v001"]
    assert result.scores == {}


def test_pareto_select_projects_superset_baseline():
    # Baseline measures an extra dim the variant lacks; projection to the active
    # dim keeps them comparable instead of raising on a dim-set mismatch.
    selector = ParetoSelector(dimensions=("hit-rate",))
    result = selector.select([
        _vec("baseline", **{"hit-rate": (0.5, True), "per-op-time": (10.0, False)}),
        _vec("v001", **{"hit-rate": (0.6, True)}),
    ])
    assert result.winners == ["v001"]


# --- weighted selector --------------------------------------------------------


def test_weighted_single_dim_ranks_by_that_dim():
    selector = WeightedSelector(winners_count=2, weights={"hit-rate": 1.0})
    result = selector.select([
        _vec("v001", **{"hit-rate": (0.4, True)}),
        _vec("v002", **{"hit-rate": (0.9, True)}),
        _vec("v003", **{"hit-rate": (0.6, True)}),
    ])
    # min-max over {0.4, 0.9, 0.6}: v002=1.0, v003=0.4, v001=0.0
    assert result.winners == ["v002", "v003"]
    assert result.scores["v002"] == pytest.approx(1.0)
    assert result.scores["v001"] == pytest.approx(0.0)


def test_weighted_respects_direction_flip_for_lower_is_better():
    selector = WeightedSelector(winners_count=1, weights={"per-op-time": 1.0})
    result = selector.select([
        _vec("v001", **{"per-op-time": (100.0, False)}),
        _vec("v002", **{"per-op-time": (10.0, False)}),
    ])
    # Lower time is better, so the fastest wins.
    assert result.winners == ["v002"]


def test_weighted_combines_dimensions_by_weight():
    selector = WeightedSelector(winners_count=1, weights={"hit-rate": 0.9, "per-op-time": 0.1})
    result = selector.select([
        # v001: best hit-rate, slowest. v002: worst hit-rate, fastest.
        _vec("v001", **{"hit-rate": (0.9, True), "per-op-time": (100.0, False)}),
        _vec("v002", **{"hit-rate": (0.4, True), "per-op-time": (10.0, False)}),
    ])
    # hit-rate dominates the weighting, so v001 wins despite being slow.
    assert result.winners == ["v001"]


def test_weighted_degenerate_dimension_contributes_zero():
    # Every variant ties on the constant dim; it must not divide by a zero spread.
    selector = WeightedSelector(winners_count=2, weights={"const": 1.0})
    result = selector.select([
        _vec("v001", const=(5.0, True)),
        _vec("v002", const=(5.0, True)),
    ])
    assert result.scores == {"v001": 0.0, "v002": 0.0}
    assert set(result.winners) == {"v001", "v002"}


def test_weighted_clamps_winners_count_to_population():
    selector = WeightedSelector(winners_count=10, weights={"hit-rate": 1.0})
    result = selector.select([_vec("v001", **{"hit-rate": (0.5, True)})])
    assert result.winners == ["v001"]


def test_weighted_empty_returns_empty():
    selector = WeightedSelector(winners_count=3, weights={"hit-rate": 1.0})
    result = selector.select([])
    assert result.winners == []
    assert result.scores == {}
