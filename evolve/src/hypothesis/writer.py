"""Drives the hypotheses-writer agent and verifies its output."""

from __future__ import annotations

import logging
import re
from pathlib import Path

from src._io import write_utf8
from src.adapters import LLMAdapter
from src.identifiers import HYPOTHESIS_ID_WIDTH, CycleId
from src.paths import Layout

AGENT_NAME = "hypotheses-writer"
HYPOTHESIS_FILE_RE = re.compile(rf"H\d{{{HYPOTHESIS_ID_WIDTH}}}\.md")
COLD_START_CYCLE = CycleId.cold_start()

log = logging.getLogger("hypotheses-writer")


def _task_md_header(cycle_id: CycleId, hypotheses_num: int, hypotheses_dir: Path, *, title_suffix: str = "") -> str:
    return (
        f"# Task: hypotheses-writer (cycle {cycle_id}{title_suffix})\n\n"
        f"Cycle: {cycle_id}\n"
        f"Number of hypotheses: {hypotheses_num}\n"
        f"Output dir: {hypotheses_dir}\n\n"
    )


def _write_cold_start_task_md(hypotheses_num: int, layout: Layout) -> None:
    cycle_layout = layout.outputs.cycle(COLD_START_CYCLE)
    baseline_layout = layout.sources.baseline
    body = _task_md_header(
        COLD_START_CYCLE, hypotheses_num, cycle_layout.hypotheses_dir,
        title_suffix=" — cold start",
    ) + (
        f"## Framing\n\n"
        f"This is the first cycle. No prior variants exist. You are proposing the "
        f"first batch of {hypotheses_num} hypotheses against the permanent research "
        f"baseline below. Prefer breadth — distinct strategies for bucket-movement — "
        f"over depth on any single idea.\n\n"
        f"Every hypothesis in this cycle MUST cite `baseline` as its parent variant. "
        f"No other parent variant is valid in cycle {COLD_START_CYCLE}.\n\n"
        f"## Read for context\n\n"
        f"- {layout.sources.extension_points_md} — authoritative seams a Variant can plug into. Segment trait is the only realised one.\n"
        f"- {baseline_layout.segment_rs} — baseline Segment implementation. The code your hypotheses propose to improve.\n"
        f"- {baseline_layout.manifest_toml} — baseline manifest (eviction policy).\n"
        f"- {baseline_layout.fitness_vector_json} — baseline's measured fitness vector. Numeric anchor your hypotheses target dimensions of.\n"
        f"- {baseline_layout.insight_md} — short prose describing what the baseline does.\n"
    )
    write_utf8(cycle_layout.hypotheses_task_md, body)


def _write_continuation_task_md(hypotheses_num: int, cycle_id: CycleId, layout: Layout) -> None:
    cycle_layout = layout.outputs.cycle(cycle_id)
    prev_cycle_id = cycle_id.prev()
    prev_cycle_layout = layout.outputs.cycle(prev_cycle_id)
    if not prev_cycle_layout.variants_dir.is_dir():
        raise FileNotFoundError(
            f"cycle {cycle_id} expects previous cycle at {prev_cycle_layout.root}, "
            f"but {prev_cycle_layout.variants_dir} is missing"
        )
    prev_variant_template = prev_cycle_layout.variant_template
    body = _task_md_header(
        cycle_id, hypotheses_num, cycle_layout.hypotheses_dir,
    ) + (
        f"## Framing\n\n"
        f"You are proposing the next batch of {hypotheses_num} hypotheses. Each "
        f"hypothesis MUST cite exactly one parent variant: either `baseline` or a past "
        f"variant `cycle/<NNNN>/v<NNN>` from any prior cycle (1..{prev_cycle_id.value}). "
        f"Prefer refining recent winners or diversifying from recent losers; aim for "
        f"distinct strategies — near-duplicates collapse the cohort's Pareto front.\n\n"
        f"## Read for context\n\n"
        f"### Previous cycle ({prev_cycle_id})\n\n"
        f"- {prev_cycle_layout.variants_dir} — every variant directory `v<NNN>/` under this path. For each variant read:\n"
        f"  - {prev_variant_template.segment_rs} — the Rust source the variant tried.\n"
        f"  - {prev_variant_template.manifest_toml} — the variant's declared eviction policy.\n"
        f"  - {prev_variant_template.fitness_vector_json} — its measured fitness vector.\n"
        f"  - {prev_variant_template.insight_md} — one-line fitness summary.\n"
        f"- {prev_cycle_layout.winners_json} — Pareto front + per-variant fitness vectors for the cycle.\n"
        f"- {prev_cycle_layout.cycle_insight_md} — cycle-level summary.\n\n"
        f"### Cross-cycle history\n\n"
        f"- {layout.outputs.leaderboard_md} — current Pareto-front variants and fitness summaries across all cycles.\n"
        f"- {layout.outputs.ledger} — one JSON row per completed cycle (cohort size, Pareto front, insights).\n"
        f"- {layout.outputs.cross_cycle_synthesis_md} — long-running narrative of what has been learned across cycles. Stub today; read if present.\n\n"
        f"### Baseline + extension points\n\n"
        f"- {layout.sources.baseline.root} — the permanent baseline (always a valid parent variant).\n"
        f"- {layout.sources.extension_points_md} — authoritative seams a Variant can plug into. Segment trait is the only realised one.\n"
    )
    write_utf8(cycle_layout.hypotheses_task_md, body)


def write_hypotheses(
    hypotheses_num: int,
    *,
    layout: Layout,
    cycle_id: CycleId,
    adapter: LLMAdapter,
) -> list[Path]:
    """Produce `hypotheses_num` H<NN>.md files; return their paths.

    Raises:
        ValueError: `hypotheses_num` < 1, or adapter wrote the wrong file count.
    """

    if hypotheses_num < 1:
        raise ValueError(f"hypotheses_num must be >= 1, got {hypotheses_num}")

    cycle_layout = layout.outputs.cycle(cycle_id)
    hypotheses_dir = cycle_layout.hypotheses_dir
    hypotheses_dir.mkdir(parents=True, exist_ok=True)
    log.info("generating %d hypotheses for cycle %s into %s", hypotheses_num, cycle_id, hypotheses_dir)

    if cycle_id == COLD_START_CYCLE:
        _write_cold_start_task_md(hypotheses_num, layout)
    else:
        _write_continuation_task_md(hypotheses_num, cycle_id, layout)
    log.debug("wrote task spec: %s", cycle_layout.hypotheses_task_md)

    adapter.run(
        agent_name=AGENT_NAME,
        workspace=hypotheses_dir,
        readable_roots=[layout.outputs_root, layout.sources.dash_src, layout.sources.subject_dir],
        allow_edits=True,
    )

    entries = sorted(p for p in hypotheses_dir.iterdir() if p.is_file())
    files = [p for p in entries if HYPOTHESIS_FILE_RE.fullmatch(p.name)]
    if len(files) != hypotheses_num:
        agent_log = hypotheses_dir / f"{AGENT_NAME}.log.md"
        other = [p.name for p in entries if not HYPOTHESIS_FILE_RE.fullmatch(p.name)]
        raise ValueError(
            f"adapter wrote {len(files)} H<NN>.md files in {hypotheses_dir}, expected {hypotheses_num}; "
            f"matched={[p.name for p in files]}; other_files={other}; agent_log={agent_log}"
        )
    log.info("adapter produced %d hypothesis files in %s", len(files), hypotheses_dir)
    return files
