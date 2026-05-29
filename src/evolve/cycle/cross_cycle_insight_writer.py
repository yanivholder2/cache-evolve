"""Writes `outputs/cross-cycle-insights.md` via the cross-cycle-insight-writer agent.

A derived view of the ledger — recomputable from scratch on any cycle.
Agent-side contract lives in `src/evolve/agents/cross-cycle-insight-writer.md`.
"""

from __future__ import annotations

import logging
import shutil

from evolve._io import read_utf8, write_utf8
from evolve.llm import LLMAdapter
from evolve.cycle.ledger_digest import digest_all_cycles
from evolve.paths import CycleLayout, Layout

AGENT_NAME = "cross-cycle-insight-writer"
OUTPUT_FILENAME = "cross-cycle-insights.md"
WORKSPACE_NAME = "cross-cycle"

log = logging.getLogger(__name__)


SECTION_TEMPLATE = (
    "## Recurring wins\n\n"
    "- <pattern that has held up across multiple cycles>\n\n"
    "## Recurring failures\n\n"
    "- <approach that keeps not paying off, and why>\n\n"
    "## Open questions\n\n"
    "- <thread worth probing in upcoming cycles>\n\n"
    "## Per-policy notes\n\n"
    "- <observations grouped by eviction policy>\n"
)


def _write_task_md(workspace, ledger_body: str) -> None:
    body = (
        f"# Task: cross-cycle-insight-writer\n\n"
        f"## Ledger (all cycles, cumulative)\n\n"
        f"{ledger_body}"
        f"## Write\n\n"
        f"Exactly one file: `{OUTPUT_FILENAME}` in the output dir. Use the "
        f"following four fixed sections — preserve names and order:\n\n"
        f"{SECTION_TEMPLATE}"
    )
    write_utf8(workspace / "task.md", body)


def write_cross_cycle_insight(
    cycle_layout: CycleLayout,
    *,
    layout: Layout,
    adapter: LLMAdapter,
) -> None:
    """Regenerate `outputs/cross-cycle-insights.md` from the full ledger.

    Workspace is `<cycle_layout>/cross-cycle/` so the per-cycle task.md and
    agent log get sealed with the cycle. The produced file is then copied
    to the campaign-scoped destination.

    Raises on adapter failure, missing output, or empty/whitespace output.
    """

    workspace = cycle_layout.root / WORKSPACE_NAME
    workspace.mkdir(parents=True, exist_ok=True)
    ledger_body = digest_all_cycles(layout.outputs.ledger)
    _write_task_md(workspace, ledger_body)

    adapter.run(
        agent_name=AGENT_NAME,
        workspace=workspace,
        readable_roots=[layout.outputs.ledger],
        allow_edits=True,
    )

    produced = workspace / OUTPUT_FILENAME
    if not produced.is_file():
        raise FileNotFoundError(f"adapter did not produce {OUTPUT_FILENAME} in {workspace}")
    if not read_utf8(produced).strip():
        raise ValueError(f"adapter produced empty {OUTPUT_FILENAME} in {workspace}")

    destination = layout.outputs.cross_cycle_insights_md
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(produced, destination)
    log.info("cross-cycle insights written to %s", destination)
