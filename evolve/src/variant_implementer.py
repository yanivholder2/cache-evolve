"""Materializes Variant impls from Hypotheses via the variant-implementer agent.

The agent-side contract lives in `evolve/src/agents/variant-implementer.md`.
"""

from __future__ import annotations

import concurrent.futures
import logging
import sys
from dataclasses import dataclass
from pathlib import Path

from src import hypothesis_parser
from src.adapters import LLMAdapter
from src.hypothesis_parser import ParentVariant
from src.identifiers import HypothesisId, VariantId
from src.paths import CycleLayout, Layout, VariantLayout


@dataclass(frozen=True)
class ImplementerJob:
    id: VariantId
    hypothesis_file: Path
    variant_layout: VariantLayout
    layout: Layout
    adapter: LLMAdapter

AGENT_NAME = "variant-implementer"
REQUIRED_OUTPUTS = ("segment.rs", "manifest.toml")

log = logging.getLogger("variant-implementer")


def _write_task_md(
    variant_layout: VariantLayout,
    hypothesis_file: Path,
    parent_dir: Path,
    parent_variant: ParentVariant,
    layout: Layout,
) -> None:
    parent_layout = VariantLayout(root=parent_dir)
    body = (
        f"# Task: variant-implementer\n\n"
        f"Hypothesis: {hypothesis_file}\n"
        f"Parent variant: {parent_variant} ({parent_dir})\n"
        f"Output dir: {variant_layout.impl_dir}\n\n"
        f"## Read for context\n\n"
        f"- {hypothesis_file} — your hypothesis. The `## Changes` section is your assignment.\n"
        f"- {parent_layout.segment_rs} — the parent variant's Rust source. Apply the changes against this.\n"
        f"- {parent_layout.manifest_toml} — the parent variant's manifest (eviction policy, ordinal).\n"
        f"- {parent_layout.fitness_vector_json} — the parent variant's measured fitness vector (if present).\n"
        f"- {parent_layout.insight_md} — short prose describing what the parent variant does (if present).\n"
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
    """Materialize one Variant's impl/ from a single Hypothesis. Raises on failure."""

    if not hypothesis_file.is_file():
        raise FileNotFoundError(f"hypothesis file not found: {hypothesis_file}")

    hypothesis = hypothesis_parser.parse(hypothesis_file)
    parent_dir = hypothesis.parent_variant.resolve(layout)

    impl_dir = variant_layout.impl_dir
    impl_dir.mkdir(parents=True, exist_ok=True)
    _write_task_md(variant_layout, hypothesis_file, parent_dir, hypothesis.parent_variant, layout)
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


def _run_implementer_job(job: ImplementerJob) -> tuple[str, Exception | None]:
    """Pool worker. Returns (job_id, exception) so failures don't crash the pool."""

    log.debug("%s: implementer started for hypothesis %s", job.id, job.hypothesis_file)
    try:
        implement_variant(
            job.hypothesis_file,
            job.variant_layout,
            layout=job.layout,
            adapter=job.adapter,
        )
    except Exception as exception:
        log.debug("%s: implementer failed: %s", job.id, exception)
        return job.id, exception
    log.debug("%s: implementer completed successfully", job.id)
    return job.id, None


def implement_variants(
    cycle_layout: CycleLayout,
    cohort_size: int,
    *,
    layout: Layout,
    adapter: LLMAdapter,
) -> None:
    """Invoke the variant-implementer once per Variant in parallel. Aborts on any failure."""

    log.info("running %d implementers in parallel", cohort_size)
    jobs: list[ImplementerJob] = []
    for i in range(1, cohort_size + 1):
        variant_id = VariantId(i)
        variant_layout = cycle_layout.variant(variant_id)
        variant_layout.impl_dir.mkdir(parents=True, exist_ok=True)
        jobs.append(
            ImplementerJob(
                id=variant_id,
                hypothesis_file=cycle_layout.hypothesis_file(HypothesisId(i)),
                variant_layout=variant_layout,
                layout=layout,
                adapter=adapter,
            )
        )

    failed = False
    with concurrent.futures.ThreadPoolExecutor(max_workers=cohort_size) as pool:
        for variant_id, exception in pool.map(_run_implementer_job, jobs):
            if exception is not None:
                log.error("implementer %s failed", variant_id, exc_info=exception)
                failed = True
    if failed:
        sys.exit(1)
    log.info("all %d implementers succeeded", cohort_size)
