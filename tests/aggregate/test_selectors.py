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


def test_weighted_scores_percentage_change_from_baseline():
    selector = WeightedSelector(winners_count=2, weights={"hit-rate": 1.0})
    baseline = _vec("baseline", **{"hit-rate": (0.4, True)})
    result = selector.select(
        [
            _vec("v001", **{"hit-rate": (0.9, True)}),  # +125%
            _vec("v002", **{"hit-rate": (0.6, True)}),  # +50%
            _vec("v003", **{"hit-rate": (0.3, True)}),  # -25%, regresses
        ],
        baseline=baseline,
    )
    # Scores are change vs the 0.4 anchor; v003 regressed so the floor drops it.
    assert result.winners == ["v001", "v002"]
    assert result.scores["v001"] == pytest.approx(1.25)
    assert result.scores["v002"] == pytest.approx(0.5)
    assert result.scores["v003"] == pytest.approx(-0.25)


def test_weighted_lower_is_better_improvement_is_positive():
    selector = WeightedSelector(winners_count=2, weights={"per-op-time": 1.0})
    baseline = _vec("baseline", **{"per-op-time": (50.0, False)})
    result = selector.select(
        [
            _vec("v001", **{"per-op-time": (10.0, False)}),  # faster: +80%
            _vec("v002", **{"per-op-time": (100.0, False)}),  # slower: -100%
        ],
        baseline=baseline,
    )
    # Lower time is an improvement (positive change); the slower variant regresses
    # and is dropped by the floor.
    assert result.winners == ["v001"]
    assert result.scores["v001"] == pytest.approx(0.8)
    assert result.scores["v002"] == pytest.approx(-1.0)


def test_weighted_weights_outweigh_a_regression():
    # The motivating case: a big hit-rate gain outweighs a metadata regression.
    selector = WeightedSelector(winners_count=1, weights={"hit-rate": 0.75, "metadata": 0.25})
    baseline = _vec("baseline", **{"hit-rate": (0.4, True), "metadata": (40.0, False)})
    result = selector.select(
        [
            # hit-rate 0.4->0.8 (+100%), metadata 40->80 (lower-is-better, -100%).
            _vec("v001", **{"hit-rate": (0.8, True), "metadata": (80.0, False)}),
        ],
        baseline=baseline,
    )
    # 0.75*(+1.0) + 0.25*(-1.0) = +0.5 > 0, so v001 wins despite the regression.
    assert result.winners == ["v001"]
    assert result.scores["v001"] == pytest.approx(0.5)


def test_weighted_floor_drops_variants_that_do_not_beat_baseline():
    selector = WeightedSelector(winners_count=2, weights={"hit-rate": 1.0})
    baseline = _vec("baseline", **{"hit-rate": (0.5, True)})
    result = selector.select(
        [_vec("v001", **{"hit-rate": (0.4, True)})],  # -20%, regresses
        baseline=baseline,
    )
    assert result.winners == []
    assert result.scores["v001"] == pytest.approx(-0.2)


def test_weighted_requires_a_baseline():
    selector = WeightedSelector(winners_count=1, weights={"hit-rate": 1.0})
    with pytest.raises(ValueError, match="no baseline"):
        selector.select([_vec("v001", **{"hit-rate": (0.6, True)})], baseline=None)


def test_weighted_non_positive_baseline_dimension_raises():
    selector = WeightedSelector(winners_count=1, weights={"hit-rate": 1.0})
    baseline = _vec("baseline", **{"hit-rate": (0.0, True)})
    with pytest.raises(ValueError, match="must be positive"):
        selector.select([_vec("v001", **{"hit-rate": (0.6, True)})], baseline=baseline)


def test_weighted_clamps_winners_count_to_beaters():
    selector = WeightedSelector(winners_count=10, weights={"hit-rate": 1.0})
    baseline = _vec("baseline", **{"hit-rate": (0.4, True)})
    result = selector.select(
        [_vec("v001", **{"hit-rate": (0.5, True)})], baseline=baseline
    )
    assert result.winners == ["v001"]


def test_weighted_empty_returns_empty():
    selector = WeightedSelector(winners_count=3, weights={"hit-rate": 1.0})
    result = selector.select([], baseline=None)
    assert result.winners == []
    assert result.scores == {}
