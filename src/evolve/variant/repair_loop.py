"""Self-repair sub-loop for variant implementation.

`implement_with_repair` materialises a variant attempt, runs the gates, and on
a gate failure re-invokes the implementer with the failing gate embedded in the
next attempt's `task.md` — up to a total attempt budget. Each attempt is
preserved on disk under `attempts/NN/` for audit.

An implementer *crash* (the adapter or a scope/contract check raising) is a
hard fail: it raises `VariantStepFailed("implement")` with no retry. Only gate
*failures* trigger repair.
"""

from __future__ import annotations

import logging
from pathlib import Path

from evolve.config import AuthoringScope
from evolve.identifiers import HypothesisId
from evolve.llm import LLMAdapter
from evolve.paths import CycleVariant, Layout
from evolve.variant import implementer as _implementer
from evolve.variant.gates import GatesResult, run_gates

log = logging.getLogger("repair-loop")


class VariantStepFailed(Exception):
    """Sentinel: one step in a variant's pipeline failed; abort remaining steps.

    `reason` is a short tag like "implement", "gate:contract gate (2/3 attempts)",
    "fitness", or "insight" so the cycle-level survival summary can attribute
    each rejection.
    """

    def __init__(self, reason: str):
        super().__init__(reason)
        self.reason = reason


def implement_with_repair(
    variant: CycleVariant,
    hypothesis_file: Path,
    *,
    layout: Layout,
    repo_root: Path,
    adapter: LLMAdapter,
    authoring_scope: AuthoringScope,
    hypothesis_id: HypothesisId | None = None,
    attempts: int,
) -> GatesResult:
    """Implement `variant` and run its gates, repairing on a gate failure up to
    `attempts` total attempts (initial + repairs). Returns the last
    `GatesResult` (truthy iff some attempt passed every gate).

    Raises `VariantStepFailed("implement")` if the implementer crashes.
    """

    cycle_variant_dir = variant.resolve(layout)
    prev_attempt_dir = None
    result: GatesResult | None = None
    for attempt in range(1, attempts + 1):
        try:
            _implementer.implement_variant(
                hypothesis_file,
                cycle_variant_dir,
                layout=layout,
                adapter=adapter,
                authoring_scope=authoring_scope,
                attempt=attempt,
                prev_attempt_dir=prev_attempt_dir,
            )
        except Exception as exception:
            log.warning("[%s] implement attempt %d crashed: %s", variant.variant_id, attempt, exception)
            raise VariantStepFailed("implement") from exception

        result = run_gates(
            variant, layout=layout, repo_root=repo_root, hypothesis=hypothesis_id, attempt=attempt
        )
        if result.all_passed:
            log.info("[%s] gates passed on attempt %d/%d", variant.variant_id, attempt, attempts)
            return result
        log.info(
            "[%s] gate %r failed on attempt %d/%d",
            variant.variant_id, result.first_failed_gate, attempt, attempts,
        )
        prev_attempt_dir = cycle_variant_dir.attempt(attempt)

    return result
