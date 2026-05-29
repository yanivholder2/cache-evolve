"""The `Gate` protocol and its `GateOutcome` return type.

A gate is a pluggable correctness check run against a variant overlay. The
runner (`evolve.variant.gates.run_gates`) executes a list of gates in order,
threading a shared `ctx` dict between them and writing rejections itself.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Protocol

from evolve.paths import Layout, Variant


@dataclass(frozen=True)
class GateOutcome:
    """Outcome of running one gate.

    `rejection_body` is the gate-specific entry body (no header); the runner
    prepends the shared header and appends it to the variant's rejection.md
    when `passed` is False. `shared` carries artifacts forward to later gates —
    the build gate hands its compiled workspace to the cargo-test gates this
    way."""

    passed: bool
    rejection_body: str = ""
    shared: dict[str, Any] = field(default_factory=dict)


class Gate(Protocol):
    """A correctness check run against a variant overlay.

    `ctx` is the runner's shared dict, populated by earlier gates' `shared`
    (e.g. the compiled `workspace`). A gate never writes rejection.md or cleans
    the workspace; it returns a `GateOutcome` and lets the runner do both."""

    name: str

    def check(
        self,
        variant: Variant,
        *,
        layout: Layout,
        repo_root: Path,
        ctx: dict[str, Any],
    ) -> GateOutcome: ...
