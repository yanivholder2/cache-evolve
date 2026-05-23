"""Typed identifiers for evolve framework entities.

Each identifier is a frozen value object wrapping a 1-based ordinal that
formats to a canonical on-disk string. Pass value objects around rather
than the formatted string so the format invariant lives in one place.
"""

from __future__ import annotations

from dataclasses import dataclass

CYCLE_ID_WIDTH = 4
VARIANT_ID_WIDTH = 3
HYPOTHESIS_ID_WIDTH = 2


@dataclass(frozen=True, order=True)
class CycleId:
    """A cycle's 1-based ordinal."""

    value: int

    def __post_init__(self) -> None:
        if self.value < 1:
            raise ValueError(f"CycleId must be >= 1, got {self.value}")

    @classmethod
    def from_str(cls, s: str) -> CycleId:
        """Construct from the on-disk zero-padded form."""
        if len(s) != CYCLE_ID_WIDTH or not s.isdigit():
            raise ValueError(
                f"malformed cycle id {s!r}; expected {CYCLE_ID_WIDTH}-digit zero-padded"
            )
        return cls(int(s))

    @classmethod
    def cold_start(cls) -> CycleId:
        return cls(1)

    def next(self) -> CycleId:
        return CycleId(self.value + 1)

    def prev(self) -> CycleId:
        return CycleId(self.value - 1)

    def __str__(self) -> str:
        """Zero-padded 4-digit form (e.g. `0001`); used as `cycles/<NNNN>`."""
        return f"{self.value:0{CYCLE_ID_WIDTH}d}"


@dataclass(frozen=True, order=True)
class VariantId:
    """A variant's 1-based ordinal within a cycle."""

    value: int

    def __post_init__(self) -> None:
        if self.value < 1:
            raise ValueError(f"VariantId must be >= 1, got {self.value}")

    @classmethod
    def from_str(cls, s: str) -> VariantId:
        """Construct from the on-disk `v<NNN>` form."""
        if not s.startswith("v") or len(s) != VARIANT_ID_WIDTH + 1 or not s[1:].isdigit():
            raise ValueError(
                f"malformed variant id {s!r}; expected 'v' + {VARIANT_ID_WIDTH}-digit zero-padded"
            )
        return cls(int(s[1:]))

    def __str__(self) -> str:
        """`v` + zero-padded 3-digit form (e.g. `v001`); used as `variants/v<NNN>`."""
        return f"v{self.value:0{VARIANT_ID_WIDTH}d}"


@dataclass(frozen=True, order=True)
class HypothesisId:
    """A hypothesis's 1-based ordinal."""

    value: int

    def __post_init__(self) -> None:
        if self.value < 1:
            raise ValueError(f"HypothesisId must be >= 1, got {self.value}")

    def __str__(self) -> str:
        """`H` + zero-padded 2-digit form (e.g. `H01`); used as `hypotheses/H<NN>.md`."""
        return f"H{self.value:0{HYPOTHESIS_ID_WIDTH}d}"
