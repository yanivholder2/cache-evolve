"""Variant-implementer step — single-variant driver + cohort fan-out.

Owns everything related to materializing Variant impls in a cycle:

- `implement_variant`: invokes the LLMAdapter for one Hypothesis, verifies
  the agent produced `segment.rs` and `manifest.toml`.
- `fan_out`: spawns N `implement_variant` calls in parallel (one per
  Hypothesis), aborts the cycle on any failure.
- `Implementer`: callable protocol so tests can inject stubs.

The agent-side contract (what the LLM must write, constraints, failure
modes) lives in `evolve/scripts/agents/variant-implementer.md`.
"""

from __future__ import annotations

import concurrent.futures
import functools
import logging
import sys
import traceback
from pathlib import Path
from typing import Callable

from adapters import LLMAdapter

AGENT_NAME = "variant-implementer"
REQUIRED_OUTPUTS = ("segment.rs", "manifest.toml")

Implementer = Callable[..., None]

log = logging.getLogger("variant-implementer")


def _write_task_md(
    output_impl_dir: Path,
    hypothesis_file: Path,
    evolve_root: Path,
) -> None:
    body = (
        f"# Task: variant-implementer\n\n"
        f"Hypothesis: {hypothesis_file}\n"
        f"Output dir: {output_impl_dir}\n\n"
        f"## Read for context\n\n"
        f"- {hypothesis_file}\n"
        f"- {evolve_root / 'subject' / 'extension-points.md'}\n"
        f"- {evolve_root / 'tests' / 'fixtures' / 'good' / 'impl' / 'segment.rs'}\n\n"
        f"## Write\n\n"
        f"Exactly two files under the output dir: `segment.rs` and `manifest.toml`.\n"
    )
    (output_impl_dir / "task.md").write_text(body, encoding="utf-8")


def implement_variant(
    hypothesis_file: Path,
    output_impl_dir: Path,
    *,
    evolve_root: Path,
    outputs_root: Path,
    adapter: LLMAdapter,
) -> None:
    """Materialize one Variant's impl/ from a single Hypothesis.

    Raises on failure; the orchestrator catches and surfaces the exception
    per-variant. `outputs_root` is accepted for parity with peer drivers.
    """

    del outputs_root  # currently unused; reserved for future context bundles
    if not hypothesis_file.is_file():
        raise FileNotFoundError(f"hypothesis file not found: {hypothesis_file}")

    output_impl_dir.mkdir(parents=True, exist_ok=True)
    _write_task_md(output_impl_dir, hypothesis_file, evolve_root)
    log.debug("wrote %s", output_impl_dir / "task.md")

    adapter.run(agent_name=AGENT_NAME, output_dir=output_impl_dir)

    missing = [n for n in REQUIRED_OUTPUTS if not (output_impl_dir / n).is_file()]
    if missing:
        raise FileNotFoundError(
            f"adapter did not produce {missing} in {output_impl_dir}"
        )
    log.info("variant materialized at %s", output_impl_dir)


def _run_one(
    job: tuple[str, Implementer, Path, Path, Path, Path],
) -> tuple[str, BaseException | None]:
    vid, implementer, hyp_file, vdir, evolve_root, outputs_root = job
    log.debug("%s: implementer %s", vid, hyp_file)
    try:
        implementer(
            hyp_file,
            vdir / "impl",
            evolve_root=evolve_root,
            outputs_root=outputs_root,
        )
    except BaseException as exc:  # noqa: BLE001 — surface every failure to the orchestrator
        log.debug("%s: implementer failed: %s", vid, exc)
        return vid, exc
    log.debug("%s: implementer ok", vid)
    return vid, None


def fan_out(
    cycle_dir: Path,
    cohort_size: int,
    *,
    evolve_root: Path,
    outputs_root: Path,
    adapter: LLMAdapter,
    implementer: Implementer | None = None,
) -> None:
    """Invoke the variant-implementer once per Variant in parallel. Aborts on any failure.

    When `implementer` is None (production path), `implement_variant` is
    bound with the adapter via `functools.partial`. Tests can pass a stub
    `implementer=` that ignores the adapter.
    """

    if implementer is None:
        implementer = functools.partial(implement_variant, adapter=adapter)

    log.info("fan-out: %d implementer(s) in parallel", cohort_size)
    jobs = []
    for i in range(1, cohort_size + 1):
        vdir = cycle_dir / "variants" / f"v{i:03d}"
        (vdir / "impl").mkdir(parents=True, exist_ok=True)
        (vdir / "eval" / "fitness").mkdir(parents=True, exist_ok=True)
        jobs.append(
            (f"v{i:03d}", implementer, cycle_dir / "hypotheses" / f"H{i:02d}.md", vdir, evolve_root, outputs_root)
        )

    failed = False
    with concurrent.futures.ThreadPoolExecutor(max_workers=cohort_size) as pool:
        for vid, exc in pool.map(_run_one, jobs):
            if exc is not None:
                print(f"error: variant-implementer for {vid} failed: {exc}", file=sys.stderr)
                sys.stderr.write("".join(traceback.format_exception(exc)))
                log.error("implementer %s failed: %s", vid, exc)
                failed = True
    if failed:
        sys.exit(1)
    log.info("fan-out: all %d implementers succeeded", cohort_size)
