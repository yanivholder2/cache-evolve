"""Hypothesis-writer driver — invokes an LLMAdapter and verifies it wrote
exactly N H<NN>.md files.

Swap target for #64: when the curated context-bundle assembler lands, the
agent's `task.md` (assembled below) will point at the bundle instead of raw
leaderboard / ledger paths.
"""

from __future__ import annotations

import logging
import re
from pathlib import Path

from adapters import LLMAdapter
from paths import Layout

AGENT_NAME = "hypothesis-writer"
HYPOTHESIS_FILE_RE = re.compile(r"H\d{2}\.md")

log = logging.getLogger("hypothesis-writer")


def _write_task_md(hypotheses_num: int, cycle_num: str, layout: Layout) -> None:
    cycle_layout = layout.cycle(cycle_num)
    body = (
        f"# Task: hypothesis-writer (cycle {cycle_num})\n\n"
        f"N: {hypotheses_num}\n"
        f"Cycle: {cycle_num}\n"
        f"Output dir: {cycle_layout.hypotheses_dir}\n\n"
        f"## Read for context\n\n"
        f"- {layout.leaderboard_md}\n"
        f"- {layout.ledger}\n"
        f"- {layout.extension_points_md}\n\n"
        f"## Write\n\n"
        f"Exactly {hypotheses_num} files: H01.md..H{hypotheses_num:02d}.md under the output dir.\n"
        f"Each file's first non-empty non-header line is the canonical "
        f"Hypothesis text. Files must be distinct.\n"
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
    log.info("hypotheses_num=%d cycle=%s output_dir=%s", hypotheses_num, cycle_num, output_dir)

    _write_task_md(hypotheses_num, cycle_num, layout)
    log.debug("wrote %s", cycle_layout.hypotheses_task_md)

    adapter.run(agent_name=AGENT_NAME, output_dir=output_dir)

    files = sorted(p for p in output_dir.iterdir() if HYPOTHESIS_FILE_RE.fullmatch(p.name))
    if len(files) != hypotheses_num:
        raise ValueError(
            f"adapter wrote {len(files)} H<NN>.md files in {output_dir}, expected {hypotheses_num}"
        )
    log.info("wrote %d H<NN>.md files in %s", len(files), output_dir)
    return files
