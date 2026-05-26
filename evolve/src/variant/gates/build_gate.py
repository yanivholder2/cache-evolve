"""Build gate: variant cargo crate compiles with --tests."""

from __future__ import annotations

import logging
import subprocess
from pathlib import Path
from typing import Any

from src.identifiers import HypothesisId
from src.paths import Layout, Variant
from src.variant.build import build_variant_for_gates
from src.variant.gates._protocol import GateOutcome

log = logging.getLogger("build-gate")


class BuildGate:
    name = "build gate"

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
        try:
            workspace = build_variant_for_gates(variant, layout=layout, repo_root=repo_root)
        except FileNotFoundError as exception:
            log.error("%s", exception)
            return GateOutcome(
                passed=False,
                rejection_body=(
                    f"**Gate**: build gate\n"
                    f"**Hypothesis**: {hypothesis}\n"
                    f"**Error**:\n"
                    f"```\n{exception}\n```\n"
                ),
            )
        except subprocess.CalledProcessError as exception:
            log.info("build gate: FAILED for %s", variant)
            return GateOutcome(
                passed=False,
                rejection_body=(
                    f"**Gate**: build gate\n"
                    f"**Hypothesis**: {hypothesis}\n"
                    f"**Error**:\n"
                    f"```\n{(exception.stdout or '') + (exception.stderr or '')}\n```\n"
                ),
            )

        log.info("build gate: passed for %s", variant)
        return GateOutcome(passed=True, shared={"gates_workspace": workspace})
