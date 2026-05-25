"""Materializes one Variant's impl from a Hypothesis via the variant-implementer agent.

The agent-side contract lives in `evolve/src/agents/variant-implementer.md`.
"""

from __future__ import annotations

import logging
from pathlib import Path

from src._io import write_utf8
from src.hypothesis import parser
from src.adapters import LLMAdapter
from src.paths import CycleVariantDir, Layout, Variant, VariantDir

AGENT_NAME = "variant-implementer"
REQUIRED_OUTPUTS = ("segment.rs", "manifest.toml")

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
        f"Output dir: {variant_dir.impl_dir}\n\n"
        f"## Paths\n\n"
        f"- Parent segment.rs:     {parent_dir.segment_rs}\n"
        f"- Parent manifest.toml:  {parent_dir.manifest_toml}\n"
        f"- Parent fitness-vector: {parent_dir.fitness_vector_json}\n"
        f"- Parent insight.md:     {parent_dir.insight_md}\n"
        f"- Extension points:      {layout.sources.extension_points_md}\n"
        f"- Baseline segment.rs:   {layout.sources.baseline.segment_rs}\n\n"
        f"## Write\n\n"
        f"Exactly two files under the output dir: `segment.rs` and `manifest.toml`.\n"
    )
    write_utf8(variant_dir.impl_task_md, body)


def implement_variant(
    hypothesis_file: Path,
    variant_dir: CycleVariantDir,
    *,
    layout: Layout,
    adapter: LLMAdapter,
) -> None:
    """Materialize one Variant's impl/ from a single Hypothesis. Raises on failure."""

    if not hypothesis_file.is_file():
        raise FileNotFoundError(f"hypothesis file not found: {hypothesis_file}")

    hypothesis = parser.parse(hypothesis_file)
    parent_dir = hypothesis.parent_variant.resolve(layout)

    impl_dir = variant_dir.impl_dir
    impl_dir.mkdir(parents=True, exist_ok=True)
    _write_task_md(variant_dir, hypothesis_file, parent_dir, hypothesis.parent_variant, layout)
    log.debug("wrote implementer task.md at %s", variant_dir.impl_task_md)

    adapter.run(
        agent_name=AGENT_NAME,
        workspace=impl_dir,
        readable_roots=[hypothesis_file, parent_dir.root, layout.sources.dash_src, layout.sources.subject_dir],
        allow_edits=True,
    )

    missing = [n for n in REQUIRED_OUTPUTS if not (impl_dir / n).is_file()]
    if missing:
        raise FileNotFoundError(
            f"adapter did not produce {missing} in {impl_dir}"
        )
    log.info("variant materialized at %s", impl_dir)
