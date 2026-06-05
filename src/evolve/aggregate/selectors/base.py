"""The selection seam: pick Winners from a cohort's fitness vectors.

A `Selector` turns a list of fitness vectors into a `SelectionResult` (the
Winners plus, where the method computes one, a per-variant scalar score). The
method is pluggable — `pareto` and `weighted` ship as built-ins, resolved by
name from `SELECTOR_REGISTRY`. Each method validates its own config in
`from_config`; `select` is method-blind so the call sites never branch on it.
"""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass


class SelectorConfigError(ValueError):
    """Raised when a selector's method-specific config is invalid.

    Raised at config-load time (config.py wraps it as a ConfigError), so a bad
    `[selection]`/`[fitness]` pairing dies before cycle 0 rather than mid-run.
    """


@dataclass(frozen=True)
class SelectionResult:
    """The outcome of one selection.

    `winners` is the selected variant ids (the front for `pareto`, the top-k for
    `weighted`); its shape is method-independent so downstream code is unchanged.
    `scores` maps every variant id to its scalar score, or is empty when the
    method computes no scalar (`pareto` returns `{}` rather than fabricate one).
    """

    winners: list[str]
    scores: dict[str, float]


class Selector(ABC):
    """A pluggable Winner-selection method over a cohort's fitness vectors."""

    @classmethod
    @abstractmethod
    def from_config(
        cls,
        *,
        winners_count: int,
        dimensions: tuple[str, ...],
        weights: dict[str, float] | None,
    ) -> "Selector":
        """Build a selector from validated config, or raise `SelectorConfigError`
        when this method's config is inconsistent (e.g. weights present under
        `pareto`, or weight keys not matching the active dimensions)."""

    @abstractmethod
    def select(
        self, fitness_vectors: list[dict], *, baseline: dict | None
    ) -> SelectionResult:
        """Choose Winners from `fitness_vectors`. Each vector is projected onto
        the active dimensions first, so a baseline anchor measured on a superset
        of dimensions compares cleanly against variants measured on the subset.

        `baseline` is the subject's baseline fitness vector — the anchor
        `weighted` scores percentage change against. `pareto` is scale-invariant
        and ignores it, so it may be `None` for scale-invariant methods;
        `weighted` raises when it is missing."""


def project(fitness_vectors: list[dict], dimensions: tuple[str, ...]) -> list[dict]:
    """Return each vector with its `dimensions` restricted to `dimensions`, in
    that order. Raises if any vector lacks an active dimension — a missing active
    dim is a measurement gap, not something to silently drop."""

    active = set(dimensions)
    projected: list[dict] = []
    for fitness_vector in fitness_vectors:
        by_name = {dimension["dim"]: dimension for dimension in fitness_vector["dimensions"]}
        missing = active - by_name.keys()
        if missing:
            raise ValueError(
                f"variant {fitness_vector['variant']!r} is missing active "
                f"dimensions {sorted(missing)}; measured {sorted(by_name)}"
            )
        projected.append(
            {**fitness_vector, "dimensions": [by_name[dim] for dim in dimensions]}
        )
    return projected
