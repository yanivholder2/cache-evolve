"""The `pareto` selector: Winners are the Pareto front of the fitness vectors.

A variant **dominates** another when it is at least as good on every dimension
and strictly better on one; the front is the set nothing else dominates.
Scale-invariant, so it needs no normalization and computes no scalar score —
`select` returns empty `scores`.
"""

from __future__ import annotations

import math

from evolve.aggregate.selectors.base import (
    SelectionResult,
    Selector,
    SelectorConfigError,
    project,
)


def _score(dim: dict) -> float:
    """Normalize so higher is always better. Raises if `summary` is not finite."""

    s = float(dim["summary"])
    if not math.isfinite(s):
        raise ValueError(f"non-finite summary for dim {dim.get('dim')!r}: {s}")
    return s if dim["higher_is_better"] else -s


def _build_scores(fitness_vectors: list) -> list:
    """Return [(variant, {dim: score}), ...]; raises if dim sets differ across vectors."""

    if not fitness_vectors:
        return []
    scored = [(fv["variant"], {d["dim"]: _score(d) for d in fv["dimensions"]}) for fv in fitness_vectors]
    expected = set(scored[0][1].keys())
    for variant, scores in scored[1:]:
        if set(scores.keys()) != expected:
            raise ValueError(
                f"variant {variant!r} has dim set {sorted(scores)} but expected {sorted(expected)}; "
                f"Pareto requires uniform dimensions across all vectors"
            )
    return scored


def _dominates(scores_a: dict, scores_b: dict) -> bool:
    """True if `a` dominates `b`: at least as good in every dim, strictly better in one."""

    at_least_as_good = all(scores_a[d] >= scores_b[d] for d in scores_a)
    strictly_better = any(scores_a[d] > scores_b[d] for d in scores_a)
    return at_least_as_good and strictly_better


def compute_pareto_front(fitness_vectors: list) -> list:
    """Return variant names on the Pareto front (preserves input order)."""

    scored = _build_scores(fitness_vectors)
    front = []
    for i, (variant, scores_a) in enumerate(scored):
        dominated = any(_dominates(scores_b, scores_a) for j, (_, scores_b) in enumerate(scored) if j != i)
        if not dominated:
            front.append(variant)
    return front


class ParetoSelector(Selector):
    """Selects the Pareto front; computes no scalar score."""

    def __init__(self, dimensions: tuple[str, ...]) -> None:
        self._dimensions = dimensions

    @classmethod
    def from_config(
        cls,
        *,
        winners_count: int,
        dimensions: tuple[str, ...],
        weights: dict[str, float] | None,
    ) -> "ParetoSelector":
        if weights is not None:
            raise SelectorConfigError(
                "selection.weights must be absent under method 'pareto'; weights "
                "apply only to the 'weighted' method"
            )
        return cls(dimensions=dimensions)

    def select(self, fitness_vectors: list[dict]) -> SelectionResult:
        projected = project(fitness_vectors, self._dimensions)
        front = compute_pareto_front(projected)
        return SelectionResult(winners=front, scores={})
