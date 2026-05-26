"""Variant self-repair sub-loop: implement → gates, retry up to a budget.

On a failed gate, the variant-implementer is re-invoked with the latest
rejection.md entry embedded into the next attempt's task.md. The previous
attempt's segment/manifest serve as the parent so the agent's job is to
*fix* what failed rather than restart from the original baseline.

The loop owns the `attempts/NN/` numbering; each attempt is fully isolated
on disk. The orchestrator (evolve.py) wraps this call in a single
`implement-and-gate` pipeline step.
"""

from __future__ import annotations

import logging

from pathlib import Path

from src.adapters import LLMAdapter
from src.identifiers import HypothesisId
from src.paths import CycleVariant, CycleVariantDir, Layout
from src.variant.gate_check import GatesResult, run_gates
from src.variant.implementer import implement_variant

log = logging.getLogger("repair-loop")


class VariantStepFailed(Exception):
    """One step in a variant's pipeline failed; abort remaining steps.

    `reason` is a short tag like "implement", "gate:invariant gate (3/3 attempts)",
    "fitness", or "insight" so the cycle-level survival summary can attribute
    each rejection.
    """

    def __init__(self, reason: str):
        super().__init__(reason)
        self.reason = reason


def implement_with_repair(
    hypothesis_file: Path,
    cycle_variant_dir: CycleVariantDir,
    *,
    variant: CycleVariant,
    layout: Layout,
    repo_root: Path,
    adapter: LLMAdapter,
    max_attempts: int,
    hypothesis: HypothesisId,
) -> GatesResult:
    """Implement → gates, retrying on gate failure up to `max_attempts` total.

    Returns the GatesResult from the first attempt that passed all gates, or
    from the final attempt when the budget is exhausted (caller decides what
    to do with a still-failing result).

    Raises:
        VariantStepFailed("implement"): the implementer raised on some
            attempt. Repair won't help (different bug class from "wrong
            logic") so the loop hard-fails immediately.
    """

    if max_attempts < 1:
        raise ValueError(f"max_attempts must be >= 1, got: {max_attempts}")

    last_result: GatesResult = GatesResult()
    prev_attempt_dir = None
    for attempt in range(1, max_attempts + 1):
        try:
            attempt_dir = implement_variant(
                hypothesis_file, cycle_variant_dir,
                attempt=attempt, prev_attempt_dir=prev_attempt_dir,
                layout=layout, adapter=adapter,
            )
        except Exception as exception:
            log.warning("[%s] attempt %d implementer raised: %s", variant.variant_id, attempt, exception)
            raise VariantStepFailed("implement") from exception

        last_result = run_gates(
            variant, attempt=attempt,
            layout=layout, repo_root=repo_root, hypothesis=hypothesis,
        )
        if last_result.all_passed:
            log.info("[%s] attempt %d passed all gates", variant.variant_id, attempt)
            return last_result

        log.info(
            "[%s] attempt %d rejected at %s",
            variant.variant_id, attempt, last_result.first_failed_gate,
        )
        prev_attempt_dir = attempt_dir

    log.info(
        "[%s] repair budget exhausted after %d attempts; last failure: %s",
        variant.variant_id, max_attempts, last_result.first_failed_gate,
    )
    return last_result
