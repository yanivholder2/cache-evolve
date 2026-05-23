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
        f"Variant: {variant_layout.variant_id}\n"
        f"Parent variant: {parent_variant} ({parent_dir})\n"
        f"Hypothesis: {hypothesis_file}\n\n"
        f"## Paths\n\n"
        f"- Parent segment.rs:     {parent_layout.segment_rs}\n"
        f"- Parent manifest.toml:  {parent_layout.manifest_toml}\n"
        f"- Parent fitness-vector: {parent_layout.fitness_vector_json}\n"
        f"- Parent insight.md:     {parent_layout.insight_md}\n"
        f"- Extension points:      {layout.extension_points_md}\n"
        f"- Baseline segment.rs:   {layout.baseline.segment_rs}\n"
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
    parent_variant_dir = hypothesis.parent_variant.resolve(layout)

    impl_dir = variant_layout.impl_dir
    impl_dir.mkdir(parents=True, exist_ok=True)
    _write_task_md(variant_layout, hypothesis_file, parent_variant_dir, hypothesis.parent_variant, layout)
    log.debug("wrote implementer task.md at %s", variant_layout.impl_task_md)

    adapter.run(
        agent_name=AGENT_NAME,
        workspace=impl_dir,
        readable_roots=[hypothesis_file, parent_variant_dir, layout.dash_src, layout.subject_dir],
        allow_edits=True,
    )

    missing = [n for n in REQUIRED_OUTPUTS if not (impl_dir / n).is_file()]
    if missing:
        raise FileNotFoundError(
            f"adapter did not produce {missing} in {impl_dir}"
        )
    log.info("variant materialized at %s", impl_dir)


def _run_implementer_job(job: ImplementerJob) -> tuple[VariantId, Exception | None]:
    """Pool worker. Returns (job_id, exception) so failures don't crash the pool."""

    log.info("%s: implementer started for hypothesis %s", job.id, job.hypothesis_file)
    try:
        implement_variant(
            job.hypothesis_file,
            job.variant_layout,
            layout=job.layout,
            adapter=job.adapter,
        )
    except Exception as exception:
        log.warning("%s: implementer failed: %s", job.id, exception)
        return job.id, exception
    log.info("%s: implementer completed successfully", job.id)
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
