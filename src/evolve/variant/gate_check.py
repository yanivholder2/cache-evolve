"""Build, contract, universal-invariant, and invariant gate checks for a variant overlay."""

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
from evolve.variant.build import compile_for_gates
from evolve.paths import Layout, Variant, CargoWorkspace

log = logging.getLogger("gate-check")

INVARIANT_GATE_MARKER_RE = re.compile(r"invariant gate violated: P[1-5] [a-z_]+")
UNIVERSAL_INVARIANT_GATE_MARKER_RE = re.compile(r"universal invariant violated: U[0-9]+ [a-z_]+")


@dataclass(frozen=True)
class GatesResult:
    """Per-gate outcome of a `run_gates()` call. Gates execute in order
    build gate → contract gate → universal invariant gate → invariant gate;
    a gate that didn't run (because an earlier one failed) is recorded as False."""

    build_gate_passed: bool = False
    contract_gate_passed: bool = False
    universal_invariant_gate_passed: bool = False
    invariant_gate_passed: bool = False

    @property
    def all_passed(self) -> bool:
        return (
            self.build_gate_passed
            and self.contract_gate_passed
            and self.universal_invariant_gate_passed
            and self.invariant_gate_passed
        )

    @property
    def first_failed_gate(self) -> str | None:
        if not self.build_gate_passed:
            return "build gate"
        if not self.contract_gate_passed:
            return "contract gate"
        if not self.universal_invariant_gate_passed:
            return "universal invariant gate"
        if not self.invariant_gate_passed:
            return "invariant gate"
        return None

    def __bool__(self) -> bool:
        return self.all_passed


def _append_entry(rejection_md: Path, body: str) -> None:
    rejection_md.parent.mkdir(parents=True, exist_ok=True)
    append_utf8(rejection_md, body)


def _run_cargo_test(workspace: CargoWorkspace, test_name: str) -> subprocess.CompletedProcess:
    # RUST_BACKTRACE=1 makes panic stack traces show up in `cargo test` output.
    # Without it, rejection.md captures only the panic line, which is rarely
    # enough to explain *why* a contract test keeps failing across cycles.
    env = {**os.environ, "RUST_BACKTRACE": "1"}
    # The invariant_gate target requires the test-utils feature; contract_gate
    # doesn't, but enabling the feature for both is harmless and keeps the
    # invocation uniform.
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
    workspace: CargoWorkspace,
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


def _check_marker_gate(
    workspace: CargoWorkspace,
    variant: Variant,
    rejection_md: Path,
    hypothesis: HypothesisId | None,
    *,
    test_name: str,
    gate_name: str,
    marker_re: re.Pattern[str],
    violation_prefix: str,
) -> bool:
    """Run a marker-driven cargo gate. On failure, parse the `<gate> violated:`
    marker out of the test output to name the failing invariant, then log a
    structured rejection entry. Returns True iff the gate passed."""

    proc = _run_cargo_test(workspace, test_name)
    if proc.returncode == 0:
        log.info("%s: passed for %s", gate_name, variant)
        return True
    log.info("%s: FAILED for %s (hypothesis=%s)", gate_name, variant, hypothesis)
    output = proc.stdout + proc.stderr

    marker = marker_re.search(output)
    if marker:
        invariant = marker.group(0).removeprefix(violation_prefix)
        gate_label = gate_name
    else:
        log.warning("%s cargo test failed without a marker — treating as harness error", gate_name)
        invariant = "<harness error — see Error block>"
        gate_label = f"{gate_name} (harness error)"

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


def _check_universal_invariant_gate(
    workspace: CargoWorkspace,
    variant: Variant,
    rejection_md: Path,
    hypothesis: HypothesisId | None,
) -> bool:
    return _check_marker_gate(
        workspace, variant, rejection_md, hypothesis,
        test_name="universal_invariant_gate",
        gate_name="universal invariant gate",
        marker_re=UNIVERSAL_INVARIANT_GATE_MARKER_RE,
        violation_prefix="universal invariant violated: ",
    )


def _check_invariant_gate(
    workspace: CargoWorkspace,
    variant: Variant,
    rejection_md: Path,
    hypothesis: HypothesisId | None,
) -> bool:
    return _check_marker_gate(
        workspace, variant, rejection_md, hypothesis,
        test_name="invariant_gate",
        gate_name="invariant gate",
        marker_re=INVARIANT_GATE_MARKER_RE,
        violation_prefix="invariant gate violated: ",
    )


def run_gates(
    variant: Variant,
    *,
    layout: Layout,
    repo_root: Path,
    hypothesis: HypothesisId | None = None,
) -> GatesResult:
    """Run build gate + contract gate + universal invariant gate + invariant
    gate against the variant overlay. Result is truthy iff all pass.

    A failing gate writes its entry to `<variant_dir>/rejection.md`.
    """

    rejection_md = variant.resolve(layout).rejection_md
    try:
        workspace = compile_for_gates(
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
        universal_invariant_gate_passed = _check_universal_invariant_gate(workspace, variant, rejection_md, hypothesis)
        if not universal_invariant_gate_passed:
            return GatesResult(
                build_gate_passed=True,
                contract_gate_passed=True,
                universal_invariant_gate_passed=False,
            )
        invariant_gate_passed = _check_invariant_gate(workspace, variant, rejection_md, hypothesis)
    finally:
        shutil.rmtree(
            workspace.root,
            onexc=lambda fn, path, exception: log.warning("workspace cleanup failed for %s: %s", path, exception),
        )

    result = GatesResult(
        build_gate_passed=True,
        contract_gate_passed=True,
        universal_invariant_gate_passed=True,
        invariant_gate_passed=invariant_gate_passed,
    )
    if result.all_passed:
        log.info("gate-check: all gates passed for %s", variant)
    return result
