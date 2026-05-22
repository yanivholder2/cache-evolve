"""Hypotheses-writer driver — invokes an LLMAdapter and verifies it wrote
exactly N H<NN>.md files.

The user prompt carries invocation parameters (cycle, N, output dir) plus
the context files the agent should read, each with a one-line explanation.
Output-format rules (file naming, canonical-line shape, distinctness) live
in `evolve/scripts/agents/hypotheses-writer.md` — one source of truth.
"""

from __future__ import annotations

import logging
import re
from pathlib import Path

from adapters import LLMAdapter
from paths import Layout

AGENT_NAME = "hypotheses-writer"
HYPOTHESIS_FILE_RE = re.compile(r"H\d{2}\.md")

log = logging.getLogger("hypotheses-writer")


def _write_task_md(hypotheses_num: int, cycle_num: str, layout: Layout) -> None:
    cycle_layout = layout.cycle(cycle_num)
    body = (
        f"# Task: hypotheses-writer\n\n"
        f"Cycle: {cycle_num}\n"
        f"N: {hypotheses_num}\n"
        f"Output dir: {cycle_layout.hypotheses_dir}\n\n"
        f"## Read for context\n\n"
        f"- {layout.leaderboard_md} — current Pareto-front variants and fitness summaries. Empty/missing on cold start.\n"
        f"- {layout.ledger} — one JSON row per prior cycle (cohort size, Pareto front, insights). Empty on cold start.\n"
        f"- {layout.extension_points_md} — authoritative seams a Variant can plug into. Segment trait is the only realised one.\n"
    )
    cycle_layout.hypotheses_task_md.write_text(body, encoding="utf-8")


def write_hypotheses(
    hypotheses_num: int,
    *,
    layout: Layout,
    cycle_num: str,
    adapter: LLMAdapter,
) -> list[Path]:
    """Drive the adapter to produce N H<NN>.md files; return their paths.

    Raises ValueError on bad N or wrong file count, and propagates any
    exception raised by the adapter.
    """

    if hypotheses_num < 1:
        raise ValueError(f"N must be >= 1, got {hypotheses_num}")

    cycle_layout = layout.cycle(cycle_num)
    output_dir = cycle_layout.hypotheses_dir
    output_dir.mkdir(parents=True, exist_ok=True)
    log.info("generating %d hypotheses for cycle %s into %s", hypotheses_num, cycle_num, output_dir)

    _write_task_md(hypotheses_num, cycle_num, layout)
    log.debug("wrote task spec: %s", cycle_layout.hypotheses_task_md)

    adapter.run(agent_name=AGENT_NAME, output_dir=output_dir, read_only=False)

    files = sorted(p for p in output_dir.iterdir() if HYPOTHESIS_FILE_RE.fullmatch(p.name))
    if len(files) != hypotheses_num:
        raise ValueError(
            f"adapter wrote {len(files)} H<NN>.md files in {output_dir}, expected {hypotheses_num}"
        )
    log.info("adapter produced %d hypothesis files in %s", len(files), output_dir)
    return files
