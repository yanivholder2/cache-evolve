"""Pluggable correctness gates for a variant overlay.

Framework gates run first (build → contract → universal invariant), then a
subject's own gates declared in its `manifest.toml` `[[gates]]` tables. A
Subject adds or changes a gate by editing its manifest + `tests/` — never
framework code.
"""

from __future__ import annotations

import logging
import re
import shutil
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from evolve._io import append_utf8
from evolve.identifiers import HypothesisId
from evolve.paths import Layout, Variant
from evolve.subject import Subject, SubjectManifestError, load_subject
from evolve.variant.gates._protocol import Gate, GateOutcome
from evolve.variant.gates.build_gate import BuildGate
from evolve.variant.gates.cargo_test_gate import CargoTestGate

log = logging.getLogger(__name__)

# Universal (U#) invariants are framework-owned and apply to every subject.
# Per-subject (P#) invariants live in each subject's manifest `[[gates]]`.
UNIVERSAL_INVARIANT_GATE_MARKER_RE = re.compile(r"universal invariant violated: U[0-9]+ [a-z_]+")

FRAMEWORK_GATES: tuple[Gate, ...] = (
    BuildGate(),
    CargoTestGate("contract gate", "contract_gate"),
    CargoTestGate(
        "universal invariant gate",
        "universal_invariant_gate",
        marker_re=UNIVERSAL_INVARIANT_GATE_MARKER_RE,
    ),
)


_FRAMEWORK_GATE_NAMES = frozenset(gate.name for gate in FRAMEWORK_GATES)


def build_gate_list(subject: Subject) -> tuple[Gate, ...]:
    """The ordered gate list for `subject`: framework gates then subject gates.

    Subject gate names are keyed into `run_gates`' `outcomes`, so a name that
    collides with a framework gate would silently overwrite its outcome —
    reject it loudly instead."""
    colliding = sorted(g.name for g in subject.gates if g.name in _FRAMEWORK_GATE_NAMES)
    if colliding:
        raise SubjectManifestError(
            f"gates: name(s) {colliding} are reserved by framework gates "
            f"{sorted(_FRAMEWORK_GATE_NAMES)}; rename them in manifest.toml"
        )
    return (
        *FRAMEWORK_GATES,
        *(
            CargoTestGate(g.name, g.test, re.compile(g.marker) if g.marker else None)
            for g in subject.gates
        ),
    )


@dataclass(frozen=True)
class GatesResult:
    """Per-gate outcomes of a `run_gates()` call, in execution order. Gates
    that never ran (an earlier one failed) are simply absent from `outcomes`."""

    outcomes: dict[str, GateOutcome]

    @property
    def all_passed(self) -> bool:
        return bool(self.outcomes) and all(outcome.passed for outcome in self.outcomes.values())

    @property
    def first_failed_gate(self) -> str | None:
        for name, outcome in self.outcomes.items():
            if not outcome.passed:
                return name
        return None

    def __bool__(self) -> bool:
        return self.all_passed


def _rejection_header(variant: Variant, hypothesis: HypothesisId | None, attempt: int) -> str:
    """Per-entry header so the source hypothesis and attempt are always
    recoverable from rejection.md alone."""
    ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    hypothesis_part = f"hypothesis={hypothesis} " if hypothesis is not None else ""
    return f"\n## {ts} — variant={variant} {hypothesis_part}attempt={attempt}\n\n"


def _append_entry(rejection_md: Path, body: str) -> None:
    rejection_md.parent.mkdir(parents=True, exist_ok=True)
    append_utf8(rejection_md, body)


def append_rejection(
    rejection_md: Path,
    *,
    variant: Variant,
    hypothesis: HypothesisId | None,
    attempt: int,
    body: str,
) -> None:
    """Append a rejection entry (header + `body`) to `rejection_md`, single-sourcing
    the entry format shared by gate failures and implementer rejections."""
    _append_entry(rejection_md, _rejection_header(variant, hypothesis, attempt) + body)


def run_gates(
    variant: Variant,
    *,
    layout: Layout,
    repo_root: Path,
    hypothesis: HypothesisId | None = None,
    attempt: int = 1,
) -> GatesResult:
    """Run the subject's gate list against the variant overlay. Result is
    truthy iff every gate passed; the first failing gate stops the run.

    A failing gate's entry (header + body) is appended to the variant's
    rejection.md. The build gate's compiled workspace is shared with later
    gates and removed here in a `finally`.
    """

    subject = load_subject(layout.sources.subject_dir)
    gates = build_gate_list(subject)
    rejection_md = variant.resolve(layout).rejection_md
    outcomes: dict[str, GateOutcome] = {}
    ctx: dict[str, Any] = {}
    try:
        for gate in gates:
            outcome = gate.check(variant, layout=layout, repo_root=repo_root, ctx=ctx)
            outcomes[gate.name] = outcome
            ctx.update(outcome.shared)
            if not outcome.passed:
                append_rejection(
                    rejection_md,
                    variant=variant,
                    hypothesis=hypothesis,
                    attempt=attempt,
                    body=outcome.rejection_body,
                )
                log.info("%s: FAILED for %s; rejection logged to %s", gate.name, variant, rejection_md)
                break
    finally:
        workspace = ctx.get("workspace")
        if workspace is not None:
            shutil.rmtree(
                workspace.root,
                onexc=lambda fn, path, exception: log.warning(
                    "workspace cleanup failed for %s: %s", path, exception
                ),
            )

    result = GatesResult(outcomes=outcomes)
    if result.all_passed:
        log.info("gate-check: all gates passed for %s", variant)
    return result


__all__ = [
    "Gate",
    "GateOutcome",
    "GatesResult",
    "BuildGate",
    "CargoTestGate",
    "FRAMEWORK_GATES",
    "UNIVERSAL_INVARIANT_GATE_MARKER_RE",
    "append_rejection",
    "build_gate_list",
    "run_gates",
]
