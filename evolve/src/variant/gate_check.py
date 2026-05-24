"""G1 + G2 + G3 gates for a variant impl/."""

from __future__ import annotations

import logging
import re
import shutil
import subprocess
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

from src._io import append_utf8
from src.variant.build import build_variant_for_gates
from src.paths import GatesWorkspace, Layout, Variant

log = logging.getLogger("gate-check")

G3_MARKER_RE = re.compile(r"G3 invariant violated: P[1-5] [a-z_]+")


@dataclass(frozen=True)
class GatesResult:
    """Per-gate outcome of a `run_gates()` call. Gates execute in order
    G1 → G2 → G3; a gate that didn't run (because an earlier one failed)
    is recorded as False."""

    g1_passed: bool = False
    g2_passed: bool = False
    g3_passed: bool = False

    @property
    def all_passed(self) -> bool:
        return self.g1_passed and self.g2_passed and self.g3_passed

    def __bool__(self) -> bool:
        return self.all_passed


def _append_entry(rejection_md: Path, body: str) -> None:
    rejection_md.parent.mkdir(parents=True, exist_ok=True)
    append_utf8(rejection_md, body)


def _run_cargo_test(workspace: GatesWorkspace, test_name: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        ["cargo", "test", "--manifest-path", str(workspace.cargo_toml), "--test", test_name],
        capture_output=True,
        text=True,
    )


def _check_g2(workspace: GatesWorkspace, variant: Variant, rejection_md: Path) -> bool:
    proc = _run_cargo_test(workspace, "g2_contract")
    if proc.returncode == 0:
        log.info("G2: passed")
        return True
    log.info("G2: FAILED")
    ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    _append_entry(
        rejection_md,
        f"\n## {ts} — variant={variant}\n\n"
        f"**Gate**: G2 (trait contract)\n"
        f"**Error**:\n"
        f"```\n{proc.stdout + proc.stderr}\n```\n"
        f"---\n",
    )
    log.info("rejection logged to %s", rejection_md)
    return False


def _check_g3(
    workspace: GatesWorkspace,
    variant: Variant,
    rejection_md: Path,
    fingerprint: str,
) -> bool:
    proc = _run_cargo_test(workspace, "g3_invariant")
    if proc.returncode == 0:
        log.info("G3: passed")
        return True
    log.info("G3: FAILED")
    output = proc.stdout + proc.stderr

    marker = G3_MARKER_RE.search(output)
    if marker:
        invariant = marker.group(0).removeprefix("G3 invariant violated: ")
        gate_label = "G3 (invariant suite)"
    else:
        log.warning("G3 cargo test failed without an invariant marker — treating as harness error")
        invariant = "<harness error — see Error block>"
        gate_label = "G3 (harness error)"

    ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    _append_entry(
        rejection_md,
        f"\n## {ts} — variant={variant}\n\n"
        f"**Fingerprint**: {fingerprint}\n"
        f"**Gate**: {gate_label}\n"
        f"**Failing invariant**: {invariant}\n"
        f"**Error**:\n"
        f"```\n{output}\n```\n"
        f"---\n",
    )
    log.info("rejection logged to %s", rejection_md)
    return False


def run_gates(
    variant: Variant,
    *,
    layout: Layout,
    repo_root: Path,
    fingerprint: str = "pending",
) -> GatesResult:
    """Run G1+G2+G3 against the variant's impl/. Result is truthy iff all pass.

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
        return GatesResult(g1_passed=False)
    except subprocess.CalledProcessError:
        return GatesResult(g1_passed=False)

    try:
        g2_passed = _check_g2(workspace, variant, rejection_md)
        if not g2_passed:
            return GatesResult(g1_passed=True, g2_passed=False)
        g3_passed = _check_g3(workspace, variant, rejection_md, fingerprint)
    finally:
        shutil.rmtree(
            workspace.root,
            onexc=lambda fn, path, exception: log.warning("workspace cleanup failed for %s: %s", path, exception),
        )

    result = GatesResult(g1_passed=True, g2_passed=True, g3_passed=g3_passed)
    if result.all_passed:
        log.info("gate-check: all gates passed for %s", variant)
    return result
