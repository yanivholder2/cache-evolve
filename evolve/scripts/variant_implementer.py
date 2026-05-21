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
from paths import CycleLayout, Layout, VariantLayout

AGENT_NAME = "variant-implementer"
REQUIRED_OUTPUTS = ("segment.rs", "manifest.toml")

Implementer = Callable[..., None]

log = logging.getLogger("variant-implementer")


def _write_task_md(
    variant_layout: VariantLayout,
    hypothesis_file: Path,
    layout: Layout,
) -> None:
    body = (
        f"# Task: variant-implementer\n\n"
        f"Hypothesis: {hypothesis_file}\n"
        f"Output dir: {variant_layout.impl_dir}\n\n"
        f"## Read for context\n\n"
        f"- {hypothesis_file}\n"
        f"- {layout.extension_points_md}\n"
        f"- {layout.good_segment_rs}\n\n"
        f"## Write\n\n"
        f"Exactly two files under the output dir: `segment.rs` and `manifest.toml`.\n"
    )
    variant_layout.impl_task_md.write_text(body, encoding="utf-8")


def implement_variant(
    hypothesis_file: Path,
    variant_layout: VariantLayout,
    *,
    layout: Layout,
    adapter: LLMAdapter,
) -> None:
    """Materialize one Variant's impl/ from a single Hypothesis.

    Raises on failure; the orchestrator catches and surfaces the exception
    per-variant.
    """

    if not hypothesis_file.is_file():
        raise FileNotFoundError(f"hypothesis file not found: {hypothesis_file}")

    impl_dir = variant_layout.impl_dir
    impl_dir.mkdir(parents=True, exist_ok=True)
    _write_task_md(variant_layout, hypothesis_file, layout)
    log.debug("wrote %s", variant_layout.impl_task_md)

    adapter.run(agent_name=AGENT_NAME, output_dir=impl_dir)

    missing = [n for n in REQUIRED_OUTPUTS if not (impl_dir / n).is_file()]
    if missing:
        raise FileNotFoundError(
            f"adapter did not produce {missing} in {impl_dir}"
        )
    log.info("variant materialized at %s", impl_dir)


def _run_one(
    job: tuple[str, Implementer, Path, VariantLayout, Layout],
) -> tuple[str, BaseException | None]:
    vid, implementer, hyp_file, variant_layout, layout = job
    log.debug("%s: implementer %s", vid, hyp_file)
    try:
        implementer(
            hyp_file,
            variant_layout,
            layout=layout,
        )
    except BaseException as exc:  # noqa: BLE001 — surface every failure to the orchestrator
        log.debug("%s: implementer failed: %s", vid, exc)
        return vid, exc
    log.debug("%s: implementer ok", vid)
    return vid, None


def fan_out(
    cycle_layout: CycleLayout,
    cohort_size: int,
    *,
    layout: Layout,
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
        variant_layout = cycle_layout.variant(i)
        variant_layout.impl_dir.mkdir(parents=True, exist_ok=True)
        variant_layout.fitness_dir.mkdir(parents=True, exist_ok=True)
        jobs.append((variant_layout.id, implementer, cycle_layout.hypothesis_file(i), variant_layout, layout))

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
