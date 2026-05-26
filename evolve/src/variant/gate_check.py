"""Gate runner: iterates `GATES` over a variant attempt."""

from __future__ import annotations

import logging
import shutil
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path

from src._io import append_utf8
from src.identifiers import HypothesisId
from src.paths import Layout, Variant
from src.variant.gates import GATES, GateOutcome

log = logging.getLogger("gate-check")


@dataclass(frozen=True)
class GatesResult:
    """Per-gate outcomes of one `run_gates()` call, keyed by gate name in
    execution order. Gates that didn't run (because an earlier one failed)
    are absent from `outcomes`."""

    outcomes: dict[str, GateOutcome] = field(default_factory=dict)

    @property
    def all_passed(self) -> bool:
        return bool(self.outcomes) and all(o.passed for o in self.outcomes.values())

    @property
    def first_failed_gate(self) -> str | None:
        for name, outcome in self.outcomes.items():
            if not outcome.passed:
                return name
        return None

    def __bool__(self) -> bool:
        return self.all_passed


def _rejection_header(variant: Variant, attempt: int, hypothesis: HypothesisId | None) -> str:
    """Per-entry header so the source attempt, variant, and hypothesis are
    always recoverable from rejection.md."""

    ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    hypothesis_part = f"hypothesis={hypothesis}" if hypothesis is not None else ""
    return f"\n## {ts} — variant={variant} attempt={attempt} {hypothesis_part}\n\n"


def _append_entry(rejection_md: Path, body: str) -> None:
    rejection_md.parent.mkdir(parents=True, exist_ok=True)
    append_utf8(rejection_md, body)


def run_gates(
    variant: Variant,
    *,
    attempt: int,
    layout: Layout,
    repo_root: Path,
    hypothesis: HypothesisId | None = None,
) -> GatesResult:
    """Run all `GATES` against the variant's latest attempt. Result is truthy
    iff every gate passed. A failing gate writes its entry to the variant's
    `rejection.md` and short-circuits the remaining gates."""

    rejection_md = variant.resolve(layout).rejection_md
    ctx: dict[str, object] = {}
    outcomes: dict[str, GateOutcome] = {}
    try:
        for gate in GATES:
            outcome = gate.run(
                variant, attempt, ctx,
                layout=layout, repo_root=repo_root, hypothesis=hypothesis,
            )
            outcomes[gate.name] = outcome
            if not outcome.passed:
                _append_entry(
                    rejection_md,
                    _rejection_header(variant, attempt, hypothesis) + outcome.rejection_body + "---\n",
                )
                log.info("rejection logged to %s", rejection_md)
                break
            ctx.update(outcome.shared)
    finally:
        workspace = ctx.get("gates_workspace")
        if workspace is not None:
            shutil.rmtree(
                workspace.root,
                onexc=lambda fn, path, exception: log.warning(
                    "workspace cleanup failed for %s: %s", path, exception,
                ),
            )

    result = GatesResult(outcomes=outcomes)
    if result.all_passed:
        log.info("gate-check: all gates passed for %s", variant)
    return result
