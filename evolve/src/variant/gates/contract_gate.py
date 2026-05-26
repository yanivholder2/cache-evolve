"""Contract gate: trait-shape and manifest sanity tests."""

from __future__ import annotations

import logging
import os
import subprocess
from pathlib import Path
from typing import Any

from src.identifiers import HypothesisId
from src.paths import GatesWorkspace, Layout, Variant
from src.variant.gates._protocol import GateOutcome

log = logging.getLogger("contract-gate")

TEST_NAME = "contract_gate"


def _run_cargo_test(workspace: GatesWorkspace, test_name: str) -> subprocess.CompletedProcess:
    # RUST_BACKTRACE=1 makes panic stack traces show up in `cargo test` output.
    # Without it, rejection.md captures only the panic line, which is rarely
    # enough to explain *why* a contract test keeps failing across cycles.
    env = {**os.environ, "RUST_BACKTRACE": "1"}
    return subprocess.run(
        ["cargo", "test", "--manifest-path", str(workspace.cargo_toml), "--test", test_name],
        capture_output=True,
        text=True,
        env=env,
    )


class ContractGate:
    name = "contract gate"

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
            log.info("contract gate: passed for %s", variant)
            return GateOutcome(passed=True)
        log.info("contract gate: FAILED for %s (hypothesis=%s)", variant, hypothesis)
        return GateOutcome(
            passed=False,
            rejection_body=(
                f"**Gate**: contract gate\n"
                f"**Hypothesis**: {hypothesis}\n"
                f"**Error**:\n"
                f"```\n{proc.stdout + proc.stderr}\n```\n"
            ),
        )
