"""Pluggable Winner-selection methods, resolved by name.

`SELECTOR_REGISTRY` mirrors `evolve.llm.ADAPTER_REGISTRY`: a config string names
the method, `build_selector` instantiates it.
"""

from evolve.aggregate.selectors.base import (
    SelectionResult,
    Selector,
    SelectorConfigError,
)
from evolve.aggregate.selectors.pareto import ParetoSelector
from evolve.aggregate.selectors.weighted import WeightedSelector

SELECTOR_REGISTRY: dict[str, type[Selector]] = {
    "pareto": ParetoSelector,
    "weighted": WeightedSelector,
}


def build_selector(
    *,
    method: str,
    winners_count: int,
    dimensions: tuple[str, ...],
    weights: dict[str, float] | None,
) -> Selector:
    """Resolve `method` against the registry and build its selector, raising
    `SelectorConfigError` for an unknown method or invalid method-specific config."""

    try:
        selector_cls = SELECTOR_REGISTRY[method]
    except KeyError:
        raise SelectorConfigError(
            f"selection.method must be one of {sorted(SELECTOR_REGISTRY)}, got: {method!r}"
        ) from None
    return selector_cls.from_config(
        winners_count=winners_count, dimensions=dimensions, weights=weights
    )


__all__ = [
    "SELECTOR_REGISTRY",
    "SelectionResult",
    "Selector",
    "SelectorConfigError",
    "ParetoSelector",
    "WeightedSelector",
    "build_selector",
]
