"""Build gate: compiles the variant workspace and hands it to later gates."""

from __future__ import annotations

import subprocess
from pathlib import Path
from typing import Any

from evolve.paths import Layout, Variant
from evolve.variant.build import compile_for_gates
from evolve.variant.gates._protocol import GateOutcome


def _build_body(output: str) -> str:
    return f"**Gate**: build gate\n" f"**Error**:\n" f"```\n{output}\n```\n" f"---\n"


class BuildGate:
    """Compiles the overlaid workspace. On success the compiled
    `CargoWorkspace` is returned in `shared["workspace"]` so the cargo-test
    gates can reuse it; the runner owns its cleanup.

    Only a `CalledProcessError` (the overlay broke compilation) is a gate
    failure the agent can repair. A non-repairable environment error from the
    harness (e.g. a missing subject/framework crate raising `FileNotFoundError`)
    propagates as a hard fail so the repair loop does not burn its attempt
    budget re-invoking the agent on something it cannot fix."""

    name = "build gate"

    def check(
        self,
        variant: Variant,
        *,
        layout: Layout,
        repo_root: Path,
        ctx: dict[str, Any],
    ) -> GateOutcome:
        try:
            workspace = compile_for_gates(variant, layout=layout, repo_root=repo_root)
        except subprocess.CalledProcessError as exception:
            output = (exception.output or "") + (exception.stderr or "")
            return GateOutcome(passed=False, rejection_body=_build_body(output))
        return GateOutcome(passed=True, shared={"workspace": workspace})
