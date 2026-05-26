"""Gate plug-in protocol.

Each gate is a class implementing `Gate.run(...)`. The runner threads a
shared `ctx` dict so gates can pass state forward (e.g. BuildGate's
GatesWorkspace) without coupling later gates to earlier ones at import.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Protocol

from src.identifiers import HypothesisId
from src.paths import Layout, Variant


@dataclass(frozen=True)
class GateOutcome:
    """One gate's result. `passed` is the only field every gate sets;
    `rejection_body` is appended to rejection.md on failure; `shared` is
    merged into the runner's `ctx` on success so the next gate can read it."""

    passed: bool
    rejection_body: str = ""
    shared: dict[str, Any] = field(default_factory=dict)


class Gate(Protocol):
    """Stateless per-cycle gate. Implementations live one-per-file under
    `evolve/src/variant/gates/` and are added to the `GATES` tuple."""

    name: str

    def run(
        self,
        variant: Variant,
        attempt: int,
        ctx: dict[str, Any],
        *,
        layout: Layout,
        repo_root: Path,
        hypothesis: HypothesisId | None,
    ) -> GateOutcome: ...
