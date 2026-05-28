"""Materializes one Variant's overlay from a Hypothesis via the variant-implementer agent.

The agent-side contract lives in `src/evolve/agents/variant-implementer.md`.
"""

from __future__ import annotations

import logging
from pathlib import Path

from evolve._io import write_utf8
from evolve.hypothesis import parser
from evolve.llm import LLMAdapter
from evolve.paths import CycleVariantDir, Layout, Variant, VariantDir

AGENT_NAME = "variant-implementer"

log = logging.getLogger("variant-implementer")


def _write_task_md(
    variant_dir: CycleVariantDir,
    hypothesis_file: Path,
    parent_dir: VariantDir,
    variant: Variant,
    layout: Layout,
) -> None:
    body = (
        f"# Task: variant-implementer\n\n"
        f"Variant: {variant_dir.variant_id}\n"
        f"Parent variant: {variant} ({parent_dir.root})\n"
        f"Hypothesis: {hypothesis_file}\n"
        f"Output dir: {variant_dir.files_dir} (overlay-by-path)\n\n"
        f"## Paths\n\n"
        f"- Parent files/:         {parent_dir.files_dir}\n"
        f"- Parent fitness-vector: {parent_dir.fitness_vector_json}\n"
        f"- Parent insight.md:     {parent_dir.insight_md}\n"
        f"- Subject src/:          {layout.sources.dash_src}\n\n"
        f"## Write\n\n"
        f"Place every changed file under `files/<path-within-subject>`; the overlay\n"
        f"is applied verbatim to a copy of the subject crate at build time.\n"
    )
    write_utf8(variant_dir.task_md, body)


def implement_variant(
    hypothesis_file: Path,
    variant_dir: CycleVariantDir,
    *,
    layout: Layout,
    adapter: LLMAdapter,
) -> None:
    """Materialize one Variant's overlay from a single Hypothesis. Raises on failure."""

    if not hypothesis_file.is_file():
        raise FileNotFoundError(f"hypothesis file not found: {hypothesis_file}")

    hypothesis = parser.parse(hypothesis_file)
    parent_dir = hypothesis.parent_variant.resolve(layout)

    files_dir = variant_dir.files_dir
    files_dir.mkdir(parents=True, exist_ok=True)
    _write_task_md(variant_dir, hypothesis_file, parent_dir, hypothesis.parent_variant, layout)
    log.debug("wrote implementer task.md at %s", variant_dir.task_md)

    adapter.run(
        agent_name=AGENT_NAME,
        workspace=files_dir,
        readable_roots=[hypothesis_file, parent_dir.root, layout.sources.dash_src, layout.sources.subject_dir],
        allow_edits=True,
    )

    if not any(files_dir.rglob("*")):
        # An empty overlay would build but be indistinguishable from the
        # baseline, so treat "agent wrote nothing" as a failure.
        raise FileNotFoundError(
            f"adapter produced no overlay files under {files_dir}"
        )
    log.info("variant materialized at %s", files_dir)
