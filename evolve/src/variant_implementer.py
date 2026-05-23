"""Variant-implementer step — single-variant driver + cohort driver.

Owns everything related to materializing Variant impls in a cycle:

- `implement_variant`: invokes the LLMAdapter for one Hypothesis, verifies
  the agent produced `segment.rs` and `manifest.toml`.
- `implement_variants`: runs `implement_variant` once per Hypothesis in
  parallel, aborts the cycle on any failure.

The agent-side contract (what the LLM must write, constraints, failure
modes) lives in `evolve/src/agents/variant-implementer.md`.
"""

from __future__ import annotations

import concurrent.futures
import logging
import sys
import traceback
from pathlib import Path

from src import hypothesis_parser
from src.adapters import LLMAdapter
from src.paths import CycleLayout, Layout, VariantLayout

AGENT_NAME = "variant-implementer"
REQUIRED_OUTPUTS = ("segment.rs", "manifest.toml")

log = logging.getLogger("variant-implementer")


def _write_task_md(
    variant_layout: VariantLayout,
    hypothesis_file: Path,
    parent_dir: Path,
    parent_id: str,
    layout: Layout,
) -> None:
    parent = VariantLayout(root=parent_dir)
    body = (
        f"# Task: variant-implementer\n\n"
        f"Hypothesis: {hypothesis_file}\n"
        f"Parent: {parent_id} ({parent_dir})\n"
        f"Output dir: {variant_layout.impl_dir}\n\n"
        f"## Read for context\n\n"
        f"- {hypothesis_file} — your hypothesis. The `## Changes` section is your assignment.\n"
        f"- {parent.segment_rs} — the parent's Rust source. Apply the changes against this.\n"
        f"- {parent.manifest_toml} — the parent's manifest (eviction policy, ordinal).\n"
        f"- {parent.fitness_vector_json} — the parent's measured fitness vector (if present).\n"
        f"- {parent.insight_md} — short prose describing what the parent does (if present).\n"
        f"- {layout.extension_points_md} — extension-point reference.\n"
        f"- {layout.baseline.segment_rs} — canonical shape/signatures for VariantSegment.\n\n"
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

    hypothesis = hypothesis_parser.parse(hypothesis_file)
    parent_dir = hypothesis_parser.resolve_parent_dir(hypothesis.parent, layout)

    impl_dir = variant_layout.impl_dir
    impl_dir.mkdir(parents=True, exist_ok=True)
    _write_task_md(variant_layout, hypothesis_file, parent_dir, hypothesis.parent, layout)
    log.debug("wrote %s", variant_layout.impl_task_md)

    adapter.run(
        agent_name=AGENT_NAME,
        workspace=impl_dir,
        readable_roots=[hypothesis_file, parent_dir, layout.dash_src, layout.subject_dir],
        read_only=False,
    )

    missing = [n for n in REQUIRED_OUTPUTS if not (impl_dir / n).is_file()]
    if missing:
        raise FileNotFoundError(
            f"adapter did not produce {missing} in {impl_dir}"
        )
    log.info("variant materialized at %s", impl_dir)


def _run_one(
    job: tuple[str, Path, VariantLayout, Layout, LLMAdapter],
) -> tuple[str, BaseException | None]:
    vid, hyp_file, variant_layout, layout, adapter = job
    log.debug("%s: implementer %s", vid, hyp_file)
    try:
        implement_variant(
            hyp_file,
            variant_layout,
            layout=layout,
            adapter=adapter,
        )
    except BaseException as exc:  # noqa: BLE001 — surface every failure to the orchestrator
        log.debug("%s: implementer failed: %s", vid, exc)
        return vid, exc
    log.debug("%s: implementer ok", vid)
    return vid, None


def implement_variants(
    cycle_layout: CycleLayout,
    cohort_size: int,
    *,
    layout: Layout,
    adapter: LLMAdapter,
) -> None:
    """Invoke the variant-implementer once per Variant in parallel. Aborts on any failure."""

    log.info("running %d implementers in parallel", cohort_size)
    jobs = []
    for i in range(1, cohort_size + 1):
        variant_layout = cycle_layout.variant(i)
        variant_layout.impl_dir.mkdir(parents=True, exist_ok=True)
        jobs.append((variant_layout.id, cycle_layout.hypothesis_file(i), variant_layout, layout, adapter))

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
    log.info("all %d implementers succeeded", cohort_size)
