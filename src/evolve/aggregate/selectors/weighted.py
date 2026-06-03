"""The `weighted` selector: Winners are the top-k by a weighted score.

Each dimension is min-max normalized across the cohort (after the direction
flip that makes higher always better), then combined as `Σ wᵢ·normᵢ`. Scores
are therefore cohort-relative — they rank variants within one selection, not
across selections (the cross-cycle union shifts the min/max as it grows). The
top `winners_count` variants by score are the Winners.
"""

from __future__ import annotations

from evolve.aggregate.selectors.base import (
    SelectionResult,
    Selector,
    SelectorConfigError,
    project,
)
from evolve.aggregate.selectors.pareto import _build_scores


class WeightedSelector(Selector):
    """Selects the top-k variants by a weighted sum of normalized dimensions."""

    def __init__(self, *, winners_count: int, weights: dict[str, float]) -> None:
        self._winners_count = winners_count
        self._weights = weights

    @classmethod
    def from_config(
        cls,
        *,
        winners_count: int,
        dimensions: tuple[str, ...],
        weights: dict[str, float] | None,
    ) -> "WeightedSelector":
        if weights is None:
            raise SelectorConfigError(
                "selection.weights is required under method 'weighted'"
            )
        if set(weights) != set(dimensions):
            raise SelectorConfigError(
                f"selection.weights keys {sorted(weights)} must exactly match "
                f"fitness.dimensions {sorted(dimensions)}"
            )
        return cls(winners_count=winners_count, weights=dict(weights))

    def select(self, fitness_vectors: list[dict]) -> SelectionResult:
        projected = project(fitness_vectors, tuple(self._weights))
        scored = _build_scores(projected)
        if not scored:
            return SelectionResult(winners=[], scores={})

        dims = list(scored[0][1])
        ranges = {
            dim: (min(s[dim] for _, s in scored), max(s[dim] for _, s in scored))
            for dim in dims
        }
        scores: dict[str, float] = {}
        for variant, flipped in scored:
            total = 0.0
            for dim in dims:
                low, high = ranges[dim]
                # Degenerate dim (all variants equal) carries no signal; a 0.0
                # contribution keeps every variant tied on it rather than dividing
                # by a zero spread.
                norm = 0.0 if high == low else (flipped[dim] - low) / (high - low)
                total += self._weights[dim] * norm
            scores[variant] = total

        order = [variant for variant, _ in scored]
        ranked = sorted(order, key=lambda variant: scores[variant], reverse=True)
        winners = ranked[: min(self._winners_count, len(ranked))]
        return SelectionResult(winners=winners, scores=scores)
