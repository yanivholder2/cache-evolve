"""Self-repair sub-loop for variant implementation.

`implement_with_repair` materialises a variant attempt, runs the gates, and on
a gate failure re-invokes the implementer with the failing gate embedded in the
next attempt's `task.md` — up to a total attempt budget. Each attempt is
preserved on disk under `attempts/NN/` for audit.

A correctable implementer rejection (empty overlay, out-of-scope write, or
protected-file write) is repaired like a gate failure: the rejection and its
guidance are logged to rejection.md and the next attempt re-prompts the agent,
consuming one attempt from the budget. An implementer *crash* (the adapter
raising for any other reason) is a hard fail: it raises
`VariantStepFailed("implement")` with no retry.
"""

from __future__ import annotations

import logging
from pathlib import Path

from evolve.config import AuthoringScope
from evolve.identifiers import HypothesisId
from evolve.llm import LLMAdapter
from evolve.paths import CycleVariant, Layout
from evolve.variant import implementer as _implementer
from evolve.variant.gates import GatesResult, append_rejection, run_gates

log = logging.getLogger(__name__)


class VariantStepFailed(Exception):
    """Sentinel: one step in a variant's pipeline failed; abort remaining steps.

    `reason` is a short tag like "implement", "gate:contract gate (2/3 attempts)",
    "fitness", or "insight" so the cycle-level survival summary can attribute
    each rejection.

    `detail` carries the underlying cause (gate body, scope path, "no overlay",
    or exception text) so the cycle-insight writer can see *why* a variant was
    rejected, not just at which stage.
    """

    def __init__(self, reason: str, detail: str | None = None):
        super().__init__(reason)
        self.reason = reason
        self.detail = detail


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
    """Implement `variant` and run its gates, repairing on a gate failure or a
    correctable implementer rejection up to `attempts` total attempts (initial +
    repairs). Returns the last `GatesResult` (truthy iff some attempt passed
    every gate).

    Raises `VariantStepFailed("implement")` if the implementer crashes, or
    `VariantStepFailed("implement:...")` if every attempt ended in an implementer
    rejection.
    """

    cycle_variant_dir = variant.resolve(layout)
    prev_attempt_dir = None
    result: GatesResult | None = None
    last_rejection: _implementer.ImplementerRejection | None = None
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
        except _implementer.ImplementerRejection as rejection:
            log.info(
                "[%s] implement attempt %d/%d rejected: %s",
                variant.variant_id, attempt, attempts, rejection.summary,
            )
            append_rejection(
                cycle_variant_dir.rejection_md,
                variant=variant,
                hypothesis=hypothesis_id,
                attempt=attempt,
                body=(
                    f"**Stage**: implement\n\n"
                    f"**Problem**: {rejection.summary}\n\n"
                    f"{rejection.guidance}\n"
                ),
            )
            prev_attempt_dir = cycle_variant_dir.attempt(attempt)
            last_rejection = rejection
            result = None
            continue
        except Exception as exception:
            log.warning("[%s] implement attempt %d crashed: %s", variant.variant_id, attempt, exception)
            raise VariantStepFailed("implement", detail=str(exception)) from exception

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

    if result is None and last_rejection is not None:
        raise VariantStepFailed(
            f"implement:{last_rejection.summary} ({attempts}/{attempts} attempts)",
            detail=last_rejection.guidance,
        )

    return result
