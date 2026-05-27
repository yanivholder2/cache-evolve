"""Build gate + contract gate + invariant gate checks for a variant impl/."""

from __future__ import annotations

import logging
import os
import re
import shutil
import subprocess
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

from evolve._io import append_utf8
from evolve.identifiers import HypothesisId
from evolve.variant.build import build_variant_for_gates
from evolve.paths import GatesWorkspace, Layout, Variant

log = logging.getLogger("gate-check")

INVARIANT_GATE_MARKER_RE = re.compile(r"invariant gate violated: P[1-5] [a-z_]+")


@dataclass(frozen=True)
class GatesResult:
    """Per-gate outcome of a `run_gates()` call. Gates execute in order
    build gate → contract gate → invariant gate; a gate that didn't run
    (because an earlier one failed) is recorded as False."""

    build_gate_passed: bool = False
    contract_gate_passed: bool = False
    invariant_gate_passed: bool = False

    @property
    def all_passed(self) -> bool:
        return self.build_gate_passed and self.contract_gate_passed and self.invariant_gate_passed

    @property
    def first_failed_gate(self) -> str | None:
        if not self.build_gate_passed:
            return "build gate"
        if not self.contract_gate_passed:
            return "contract gate"
        if not self.invariant_gate_passed:
            return "invariant gate"
        return None

    def __bool__(self) -> bool:
        return self.all_passed


def _append_entry(rejection_md: Path, body: str) -> None:
    rejection_md.parent.mkdir(parents=True, exist_ok=True)
    append_utf8(rejection_md, body)


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


def _rejection_header(variant: Variant, hypothesis: HypothesisId | None) -> str:
    """Common per-entry header so the source hypothesis is always recoverable.

    The hypothesis-to-variant mapping is 1:1 today but undocumented in logs;
    recording the hypothesis id alongside the variant id makes the link
    explicit (and survives any future 1:N change).
    """

    ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    hypothesis_part = f"hypothesis={hypothesis}" if hypothesis is not None else ""
    return f"\n## {ts} — variant={variant} {hypothesis_part}\n\n"


def _check_contract_gate(
    workspace: GatesWorkspace,
    variant: Variant,
    rejection_md: Path,
    hypothesis: HypothesisId | None,
) -> bool:
    proc = _run_cargo_test(workspace, "contract_gate")
    if proc.returncode == 0:
        log.info("contract gate: passed for %s", variant)
        return True
    log.info("contract gate: FAILED for %s (hypothesis=%s)", variant, hypothesis)
    _append_entry(
        rejection_md,
        _rejection_header(variant, hypothesis)
        + f"**Gate**: contract gate\n"
        + f"**Hypothesis**: {hypothesis}\n"
        + f"**Error**:\n"
        + f"```\n{proc.stdout + proc.stderr}\n```\n"
        + f"---\n",
    )
    log.info("rejection logged to %s", rejection_md)
    return False


def _check_invariant_gate(
    workspace: GatesWorkspace,
    variant: Variant,
    rejection_md: Path,
    hypothesis: HypothesisId | None,
) -> bool:
    proc = _run_cargo_test(workspace, "invariant_gate")
    if proc.returncode == 0:
        log.info("invariant gate: passed for %s", variant)
        return True
    log.info("invariant gate: FAILED for %s (hypothesis=%s)", variant, hypothesis)
    output = proc.stdout + proc.stderr

    marker = INVARIANT_GATE_MARKER_RE.search(output)
    if marker:
        invariant = marker.group(0).removeprefix("invariant gate violated: ")
        gate_label = "invariant gate"
    else:
        log.warning("invariant gate cargo test failed without an invariant marker — treating as harness error")
        invariant = "<harness error — see Error block>"
        gate_label = "invariant gate (harness error)"

    _append_entry(
        rejection_md,
        _rejection_header(variant, hypothesis)
        + f"**Gate**: {gate_label}\n"
        + f"**Hypothesis**: {hypothesis}\n"
        + f"**Failing invariant**: {invariant}\n"
        + f"**Error**:\n"
        + f"```\n{output}\n```\n"
        + f"---\n",
    )
    log.info("rejection logged to %s", rejection_md)
    return False


def run_gates(
    variant: Variant,
    *,
    layout: Layout,
    repo_root: Path,
    hypothesis: HypothesisId | None = None,
) -> GatesResult:
    """Run build gate + contract gate + invariant gate against the variant's impl/.
    Result is truthy iff all pass.

    A failing gate writes its entry to `<variant_dir>/rejection.md`.
    """

    rejection_md = variant.resolve(layout).rejection_md
    try:
        workspace = build_variant_for_gates(
            variant,
            layout=layout,
            repo_root=repo_root,
        )
    except FileNotFoundError as e:
        log.error("%s", e)
        return GatesResult(build_gate_passed=False)
    except subprocess.CalledProcessError:
        return GatesResult(build_gate_passed=False)

    try:
        contract_gate_passed = _check_contract_gate(workspace, variant, rejection_md, hypothesis)
        if not contract_gate_passed:
            return GatesResult(build_gate_passed=True, contract_gate_passed=False)
        invariant_gate_passed = _check_invariant_gate(workspace, variant, rejection_md, hypothesis)
    finally:
        shutil.rmtree(
            workspace.root,
            onexc=lambda fn, path, exception: log.warning("workspace cleanup failed for %s: %s", path, exception),
        )

    result = GatesResult(
        build_gate_passed=True,
        contract_gate_passed=True,
        invariant_gate_passed=invariant_gate_passed,
    )
    if result.all_passed:
        log.info("gate-check: all gates passed for %s", variant)
    return result
