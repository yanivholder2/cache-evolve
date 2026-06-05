"""The `weighted` selector: Winners are the top-k by weighted improvement over
the baseline.

Each dimension is scored as **percentage change from the baseline anchor**,
signed so an improvement is always positive:

    higher-is-better: (variant - baseline) / baseline
    lower-is-better:  (baseline - variant) / baseline

The per-variant score is `Σ wᵢ·improvementᵢ`, with weights (summing to 1) saying
which dimensions matter more. Variants are ranked by that score; the top
`winners_count` win, but only among those that **net-beat the baseline** (score
> 0) — a variant that regresses on the weighted balance is not crowned a Winner
just for being least-bad. Because the score is anchored to the baseline, 0 has a
fixed meaning (equal to the baseline) and the scores are comparable across
selections, unlike a cohort-relative normalization.
"""

from __future__ import annotations

from evolve.aggregate.selectors.base import (
    SelectionResult,
    Selector,
    SelectorConfigError,
    project,
)


class WeightedSelector(Selector):
    """Selects the top-k variants by weighted percentage improvement over the
    baseline, dropping any that do not net-beat it."""

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

    def select(
        self, fitness_vectors: list[dict], *, baseline: dict | None
    ) -> SelectionResult:
        if not fitness_vectors:
            return SelectionResult(winners=[], scores={})
        if baseline is None:
            raise ValueError(
                "method 'weighted' scores percentage change against the baseline "
                "anchor, but no baseline fitness vector was provided"
            )

        dims = tuple(self._weights)
        projected = project(fitness_vectors, dims)
        baseline_summary = _baseline_summary(project([baseline], dims)[0], dims)

        scores: dict[str, float] = {}
        for fitness_vector in projected:
            total = 0.0
            for dimension in fitness_vector["dimensions"]:
                name = dimension["dim"]
                base = baseline_summary[name]
                variant = float(dimension["summary"])
                change = (variant - base) / base
                # Flip lower-is-better dims so an improvement (lower value) reads
                # as positive change, matching the higher-is-better convention.
                improvement = change if dimension["higher_is_better"] else -change
                total += self._weights[name] * improvement
            scores[fitness_vector["variant"]] = total

        ranked = sorted(scores, key=lambda variant: scores[variant], reverse=True)
        # Only variants that net-beat the baseline (score > 0) can win; 0 is the
        # baseline itself, so a non-positive score means no net improvement.
        beat_baseline = [variant for variant in ranked if scores[variant] > 0]
        winners = beat_baseline[: self._winners_count]
        return SelectionResult(winners=winners, scores=scores)


def _baseline_summary(baseline: dict, dims: tuple[str, ...]) -> dict[str, float]:
    """Map each active dimension to the baseline's summary, raising if any anchor
    is non-positive — percentage change divides by it, so a zero (or negative)
    baseline magnitude is undefined and signals a measurement anomaly."""

    summary: dict[str, float] = {}
    for dimension in baseline["dimensions"]:
        value = float(dimension["summary"])
        if value <= 0:
            raise ValueError(
                f"baseline dimension {dimension['dim']!r} must be positive for "
                f"percentage-change scoring, got {value}"
            )
        summary[dimension["dim"]] = value
    return summary
