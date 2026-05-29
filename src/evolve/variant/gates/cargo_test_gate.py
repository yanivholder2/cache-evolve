"""A gate backed by a single `cargo test --test <name>` target.

Used for the contract gate, the universal invariant gate, and any
subject-declared gate. When a `marker_re` is given, a failure parses the
`<kind> violated: <invariant>` marker out of the test output to name the
failing invariant in the rejection entry.
"""

from __future__ import annotations

import logging
import os
import re
import subprocess
from pathlib import Path
from typing import Any

from evolve.paths import CargoWorkspace, Layout, Variant
from evolve.variant.gates._protocol import GateOutcome

log = logging.getLogger("gate-check")


def _run_cargo_test(workspace: CargoWorkspace, test_name: str) -> subprocess.CompletedProcess:
    # RUST_BACKTRACE=1 makes panic stack traces show up in `cargo test` output.
    # Without it, rejection.md captures only the panic line, which is rarely
    # enough to explain *why* a gate keeps failing across cycles. The
    # test-utils feature is required by the invariant gates and harmless for
    # the rest, so enabling it uniformly keeps the invocation simple.
    env = {**os.environ, "RUST_BACKTRACE": "1"}
    return subprocess.run(
        [
            "cargo", "test", "--features", "test-utils",
            "--manifest-path", str(workspace.cargo_toml),
            "--test", test_name,
        ],
        capture_output=True,
        text=True,
        env=env,
    )


class CargoTestGate:
    def __init__(self, name: str, test_name: str, marker_re: re.Pattern[str] | None = None) -> None:
        self.name = name
        self.test_name = test_name
        self.marker_re = marker_re

    def check(
        self,
        variant: Variant,
        *,
        layout: Layout,
        repo_root: Path,
        ctx: dict[str, Any],
    ) -> GateOutcome:
        workspace = ctx["workspace"]
        proc = _run_cargo_test(workspace, self.test_name)
        if proc.returncode == 0:
            log.info("%s: passed for %s", self.name, variant)
            return GateOutcome(passed=True)

        output = proc.stdout + proc.stderr
        log.info("%s: FAILED for %s", self.name, variant)
        if self.marker_re is None:
            body = (
                f"**Gate**: {self.name}\n"
                f"**Error**:\n"
                f"```\n{output}\n```\n"
                f"---\n"
            )
            return GateOutcome(passed=False, rejection_body=body)

        marker = self.marker_re.search(output)
        if marker:
            invariant = marker.group(0).split("violated: ", 1)[-1]
            gate_label = self.name
        else:
            log.warning("%s cargo test failed without a marker — treating as harness error", self.name)
            invariant = "<harness error — see Error block>"
            gate_label = f"{self.name} (harness error)"
        body = (
            f"**Gate**: {gate_label}\n"
            f"**Failing invariant**: {invariant}\n"
            f"**Error**:\n"
            f"```\n{output}\n```\n"
            f"---\n"
        )
        return GateOutcome(passed=False, rejection_body=body)
