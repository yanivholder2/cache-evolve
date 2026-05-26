"""Invariant gate: runs the P1–P5 invariant suite via Caffeine-free harness."""

from __future__ import annotations

import logging
import re
from pathlib import Path
from typing import Any

from src.identifiers import HypothesisId
from src.paths import Layout, Variant
from src.variant.gates._protocol import GateOutcome
from src.variant.gates.contract_gate import _run_cargo_test

log = logging.getLogger("invariant-gate")

TEST_NAME = "invariant_gate"
INVARIANT_GATE_MARKER_RE = re.compile(r"invariant gate violated: P[1-5] [a-z_]+")


class InvariantGate:
    name = "invariant gate"

    def run(
        self,
        variant: Variant,
        attempt: int,
        ctx: dict[str, Any],
        *,
        layout: Layout,
        repo_root: Path,
        hypothesis: HypothesisId | None,
    ) -> GateOutcome:
        workspace = ctx["gates_workspace"]
        proc = _run_cargo_test(workspace, TEST_NAME)
        if proc.returncode == 0:
            log.info("invariant gate: passed for %s", variant)
            return GateOutcome(passed=True)
        log.info("invariant gate: FAILED for %s (hypothesis=%s)", variant, hypothesis)
        output = proc.stdout + proc.stderr
        marker = INVARIANT_GATE_MARKER_RE.search(output)
        if marker:
            invariant = marker.group(0).removeprefix("invariant gate violated: ")
        else:
            log.warning("invariant gate cargo test failed without a marker — treating as harness error")
            invariant = "<harness error — see Error block>"
        return GateOutcome(
            passed=False,
            rejection_body=(
                f"**Gate**: invariant gate\n"
                f"**Hypothesis**: {hypothesis}\n"
                f"**Failing invariant**: {invariant}\n"
                f"**Error**:\n"
                f"```\n{output}\n```\n"
            ),
        )
