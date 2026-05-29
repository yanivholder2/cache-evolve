"""Writes one cycle's `cycle-insight.md` via the cycle-insight-writer agent.

Agent-side contract lives in `src/evolve/agents/cycle-insight-writer.md`.
"""

from __future__ import annotations

import logging
from pathlib import Path

from evolve._io import write_utf8
from evolve.llm import LLMAdapter
from evolve.cycle.ledger_digest import digest_prior_cycles
from evolve.paths import CycleLayout, CycleVariantDir

AGENT_NAME = "cycle-insight-writer"
REQUIRED_OUTPUTS = ("cycle-insight.md",)

log = logging.getLogger(__name__)


def _format_list(items: list[str]) -> str:
    if not items:
        return "(none)"
    return "\n".join(f"- {item}" for item in items)


def _format_paths(paths: list[Path]) -> str:
    if not paths:
        return "(none)"
    return "\n".join(f"- {p}" for p in paths)


def _write_task_md(
    cycle_layout: CycleLayout,
    survivors: list[CycleVariantDir],
    pareto_front: list[str],
    hypothesis_files: list[Path],
    digest_body: str,
) -> None:
    insight_paths = [s.insight_md for s in survivors]
    body = (
        f"# Task: cycle-insight-writer\n\n"
        f"Cycle: {cycle_layout.cycle_id}\n"
        f"Output dir: {cycle_layout.root}\n\n"
        f"## Pareto front\n\n"
        f"{_format_list(pareto_front)}\n\n"
        f"## Variant insights\n\n"
        f"{_format_paths(insight_paths)}\n\n"
        f"## Hypotheses (this cycle)\n\n"
        f"{_format_paths(hypothesis_files)}\n\n"
        f"## Prior cycles digest\n\n"
        f"{digest_body}"
        f"## Write\n\n"
        f"Exactly one file: `cycle-insight.md` in the output dir.\n"
    )
    write_utf8(cycle_layout.root / "task.md", body)


def write_cycle_insight(
    cycle_layout: CycleLayout,
    survivors: list[CycleVariantDir],
    pareto_front: list[str],
    *,
    hypothesis_files: list[Path],
    ledger: Path,
    adapter: LLMAdapter,
    digest_cycles: int,
) -> None:
    """Author one cycle's `cycle-insight.md`. Raises on adapter or output failure."""

    cycle_layout.root.mkdir(parents=True, exist_ok=True)
    digest_body = digest_prior_cycles(ledger, digest_cycles)
    _write_task_md(cycle_layout, survivors, pareto_front, hypothesis_files, digest_body)
    log.debug("wrote cycle-insight-writer task.md for cycle %s", cycle_layout.cycle_id)

    readable_roots: list[Path] = [s.insight_md for s in survivors] + list(hypothesis_files)
    adapter.run(
        agent_name=AGENT_NAME,
        workspace=cycle_layout.root,
        readable_roots=readable_roots,
        allow_edits=True,
    )

    missing = [n for n in REQUIRED_OUTPUTS if not (cycle_layout.root / n).is_file()]
    if missing:
        raise FileNotFoundError(
            f"adapter did not produce {missing} in {cycle_layout.root}"
        )
    log.info("cycle insight written for cycle %s", cycle_layout.cycle_id)
