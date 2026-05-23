"""G1 + G2 + G3 gates for a variant impl/.

Library: `run_gates(impl_dir, cycle_id, variant_id, *, repo_root, rejection_log,
fingerprint="pending") -> bool`. Returns True iff all gates pass. On failure,
appends a structured entry to rejection_log and returns False.
"""

from __future__ import annotations

import logging
import re
import shutil
import subprocess
from datetime import datetime, timezone
from pathlib import Path

from src.build import build_variant

log = logging.getLogger("gate-check")

G3_MARKER_RE = re.compile(r"G3 invariant violated: P[1-5] [a-z_]+")


def _append_entry(rejection_log: Path, body: str) -> None:
    rejection_log.parent.mkdir(parents=True, exist_ok=True)
    with open(rejection_log, "a", encoding="utf-8") as f:
        f.write(body)


def _run_cargo_test(work_dir: Path, test_name: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        ["cargo", "test", "--manifest-path", str(work_dir / "Cargo.toml"), "--test", test_name],
        capture_output=True,
        text=True,
    )


def _check_g2(work_dir: Path, cycle_id: str, variant_id: str, rejection_log: Path) -> bool:
    proc = _run_cargo_test(work_dir, "g2_contract")
    if proc.returncode == 0:
        log.info("G2: passed")
        return True
    log.info("G2: FAILED")
    ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    _append_entry(
        rejection_log,
        f"\n## {ts} — cycle={cycle_id} variant={variant_id}\n\n"
        f"**Gate**: G2 (trait contract)\n"
        f"**Error**:\n"
        f"```\n{proc.stdout + proc.stderr}\n```\n"
        f"---\n",
    )
    log.info("rejection logged to %s", rejection_log)
    return False


def _check_g3(
    work_dir: Path,
    cycle_id: str,
    variant_id: str,
    rejection_log: Path,
    fingerprint: str,
) -> bool:
    proc = _run_cargo_test(work_dir, "g3_invariant")
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
        rejection_log,
        f"\n## {ts} — cycle={cycle_id} variant={variant_id}\n\n"
        f"**Fingerprint**: {fingerprint}\n"
        f"**Gate**: {gate_label}\n"
        f"**Failing invariant**: {invariant}\n"
        f"**Error**:\n"
        f"```\n{output}\n```\n"
        f"---\n",
    )
    log.info("rejection logged to %s", rejection_log)
    return False


def run_gates(
    impl_dir: Path,
    cycle_id: str,
    variant_id: str,
    *,
    repo_root: Path,
    rejection_log: Path,
    fingerprint: str = "pending",
) -> bool:
    """Run G1+G2+G3 against impl_dir. Returns True iff all pass."""

    try:
        work_dir = build_variant(
            impl_dir,
            cycle_id,
            variant_id,
            repo_root=repo_root,
            rejection_log=rejection_log,
        )
    except FileNotFoundError as e:
        log.error("%s", e)
        return False
    except subprocess.CalledProcessError:
        return False

    try:
        if not _check_g2(work_dir, cycle_id, variant_id, rejection_log):
            return False
        if not _check_g3(work_dir, cycle_id, variant_id, rejection_log, fingerprint):
            return False
    finally:
        shutil.rmtree(
            work_dir,
            onexc=lambda fn, path, exception: log.warning("workspace cleanup failed for %s: %s", path, exception),
        )

    log.info("gate-check: all gates passed for %s", variant_id)
    return True
