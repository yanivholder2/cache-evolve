"""Writes one variant's `insight.md` via the variant-insight-writer agent.

Agent-side contract lives in `src/evolve/agents/variant-insight-writer.md`.
"""

from __future__ import annotations

import logging
from pathlib import Path

from evolve._io import write_utf8
from evolve.llm import LLMAdapter
from evolve.hypothesis import parser
from evolve.paths import CycleVariantDir, Layout

AGENT_NAME = "variant-insight-writer"

log = logging.getLogger("variant-insight-writer")


def _write_task_md(
    variant_dir: CycleVariantDir,
    hypothesis_file: Path,
    parent_dir,
) -> None:
    body = (
        f"# Task: variant-insight-writer\n\n"
        f"Variant: {variant_dir.variant_id}\n"
        f"Parent variant dir: {parent_dir.root}\n"
        f"Hypothesis: {hypothesis_file}\n"
        f"Output dir: {variant_dir.root}\n\n"
        f"## Paths\n\n"
        f"- Current variant dir:        {variant_dir.root}\n"
        f"  - fitness-vector.json:      {variant_dir.fitness_vector_json}\n"
        f"  - eval/:                    {variant_dir.eval_dir}\n"
        f"  - overlay/:                  {variant_dir.overlay_dir}\n"
        f"- Parent variant dir:         {parent_dir.root}\n"
        f"  - overlay/:                  {parent_dir.overlay_dir}\n"
        f"  - fitness-vector.json:      {parent_dir.fitness_vector_json}\n"
        f"  - insight.md:               {parent_dir.insight_md}\n"
        f"- Hypothesis:                 {hypothesis_file}\n\n"
        f"## Write\n\n"
        f"Exactly one file: `insight.md` in the output dir.\n"
    )
    write_utf8(variant_dir.root / "task.md", body)


def write_insight(
    variant_dir: CycleVariantDir,
    hypothesis_file: Path,
    *,
    layout: Layout,
    adapter: LLMAdapter,
) -> None:
    """Author one variant's `insight.md`. Raises on adapter or output failure."""

    hypothesis = parser.parse(hypothesis_file)
    parent_dir = hypothesis.parent_variant.resolve(layout)

    variant_dir.root.mkdir(parents=True, exist_ok=True)
    _write_task_md(variant_dir, hypothesis_file, parent_dir)
    log.debug("wrote insight-writer task.md for variant %s", variant_dir.variant_id)

    adapter.run(
        agent_name=AGENT_NAME,
        workspace=variant_dir.root,
        readable_roots=[variant_dir.root, parent_dir.root, hypothesis_file],
        allow_edits=True,
    )

    if not variant_dir.insight_md.is_file():
        raise FileNotFoundError(
            f"adapter did not produce {variant_dir.insight_md}"
        )
    log.info("variant insight written for %s", variant_dir.variant_id)
