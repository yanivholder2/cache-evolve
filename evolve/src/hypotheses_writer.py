"""Hypotheses-writer driver — invokes an LLMAdapter and verifies it wrote
exactly `hypotheses_num` H<NN>.md files.

Cycle 1 (cold start) and cycle 2+ get different task.md texts: cycle 1
grounds in the permanent baseline at `evolve/subject/baseline/`; cycle 2+
grounds in the previous cycle's variants plus cross-cycle artefacts. The
agent prompt itself (`agents/hypotheses-writer.md`) is cycle-agnostic —
all per-cycle framing lives in task.md.
"""

from __future__ import annotations

import logging
import re
from pathlib import Path

from src.adapters import LLMAdapter
from src.paths import CYCLE_ID_WIDTH, HYPOTHESIS_ID_WIDTH, Layout, VariantLayout

AGENT_NAME = "hypotheses-writer"
HYPOTHESIS_FILE_RE = re.compile(rf"H\d{{{HYPOTHESIS_ID_WIDTH}}}\.md")
COLD_START_CYCLE = f"{1:0{CYCLE_ID_WIDTH}d}"

log = logging.getLogger("hypotheses-writer")


def _task_md_header(cycle_num: str, hypotheses_num: int, hypotheses_dir: Path, *, title_suffix: str = "") -> str:
    return (
        f"# Task: hypotheses-writer (cycle {cycle_num}{title_suffix})\n\n"
        f"Cycle: {cycle_num}\n"
        f"Number of hypotheses: {hypotheses_num}\n"
        f"Output dir: {hypotheses_dir}\n\n"
    )


def _write_cold_start_task_md(hypotheses_num: int, layout: Layout) -> None:
    cycle_layout = layout.cycle(COLD_START_CYCLE)
    baseline_layout = layout.baseline
    body = _task_md_header(
        COLD_START_CYCLE, hypotheses_num, cycle_layout.hypotheses_dir,
        title_suffix=" — cold start",
    ) + (
        f"## Framing\n\n"
        f"This is the first cycle. No prior variants exist. You are proposing the "
        f"first batch of {hypotheses_num} hypotheses against the permanent research "
        f"baseline below. Prefer breadth — distinct strategies for bucket-movement — "
        f"over depth on any single idea.\n\n"
        f"Every hypothesis in this cycle MUST cite `baseline` as its parent. "
        f"No other parent is valid in cycle {COLD_START_CYCLE}.\n\n"
        f"## Read for context\n\n"
        f"- {layout.extension_points_md} — authoritative seams a Variant can plug into. Segment trait is the only realised one.\n"
        f"- {baseline_layout.segment_rs} — baseline Segment implementation. The code your hypotheses propose to improve.\n"
        f"- {baseline_layout.manifest_toml} — baseline manifest (name, eviction policy, ordinal).\n"
        f"- {baseline_layout.fitness_vector_json} — baseline's measured fitness vector. Numeric anchor your hypotheses target dimensions of.\n"
        f"- {baseline_layout.insight_md} — short prose describing what the baseline does.\n"
    )
    cycle_layout.hypotheses_task_md.write_text(body, encoding="utf-8")


def _write_continuation_task_md(hypotheses_num: int, cycle_num: str, layout: Layout) -> None:
    cycle_layout = layout.cycle(cycle_num)
    prev_num = f"{int(cycle_num) - 1:0{CYCLE_ID_WIDTH}d}"
    prev_cycle_layout = layout.cycle(prev_num)
    if not prev_cycle_layout.variants_dir.is_dir():
        raise FileNotFoundError(
            f"cycle {cycle_num} expects previous cycle at {prev_cycle_layout.root}, "
            f"but {prev_cycle_layout.variants_dir} is missing"
        )
    prev_cycle_variant_layout = VariantLayout(root=prev_cycle_layout.variants_dir / "v<NNN>")
    body = _task_md_header(
        cycle_num, hypotheses_num, cycle_layout.hypotheses_dir,
    ) + (
        f"## Framing\n\n"
        f"You are proposing the next batch of {hypotheses_num} hypotheses. Each "
        f"hypothesis MUST cite exactly one parent: either `baseline` or a past "
        f"variant `cycle/<NNNN>/v<NNN>` from any prior cycle (1..{int(cycle_num) - 1}). "
        f"Prefer refining recent winners or diversifying from recent losers; aim for "
        f"distinct strategies — near-duplicates collapse the cohort's Pareto front.\n\n"
        f"## Read for context\n\n"
        f"### Previous cycle ({prev_num})\n\n"
        f"- {prev_cycle_layout.variants_dir} — every variant directory `v<NNN>/` under this path. For each variant read:\n"
        f"  - {prev_cycle_variant_layout.segment_rs} — the Rust source the variant tried.\n"
        f"  - {prev_cycle_variant_layout.manifest_toml} — the variant's declared eviction policy.\n"
        f"  - {prev_cycle_variant_layout.fitness_vector_json} — its measured fitness vector.\n"
        f"  - {prev_cycle_variant_layout.insight_md} — one-line fitness summary.\n"
        f"- {prev_cycle_layout.winners_json} — Pareto front + per-variant fitness vectors for the cycle.\n"
        f"- {prev_cycle_layout.cycle_insight_md} — cycle-level summary.\n\n"
        f"### Cross-cycle history\n\n"
        f"- {layout.leaderboard_md} — current Pareto-front variants and fitness summaries across all cycles.\n"
        f"- {layout.ledger} — one JSON row per completed cycle (cohort size, Pareto front, insights).\n"
        f"- {layout.synthesis_md} — long-running narrative of what has been learned. Stub today; read if present.\n\n"
        f"### Baseline + extension points\n\n"
        f"- {layout.baseline.root} — the permanent baseline (always a valid parent).\n"
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
    """Drive the adapter to produce `hypotheses_num` H<NN>.md files; return their paths.

    Raises ValueError on bad `hypotheses_num` or wrong file count, and
    propagates any exception raised by the adapter.
    """

    if hypotheses_num < 1:
        raise ValueError(f"hypotheses_num must be >= 1, got {hypotheses_num}")

    cycle_layout = layout.cycle(cycle_num)
    hypotheses_dir = cycle_layout.hypotheses_dir
    hypotheses_dir.mkdir(parents=True, exist_ok=True)
    log.info("generating %d hypotheses for cycle %s into %s", hypotheses_num, cycle_num, hypotheses_dir)

    if cycle_num == COLD_START_CYCLE:
        _write_cold_start_task_md(hypotheses_num, layout)
    else:
        _write_continuation_task_md(hypotheses_num, cycle_num, layout)
    log.debug("wrote task spec: %s", cycle_layout.hypotheses_task_md)

    adapter.run(
        agent_name=AGENT_NAME,
        workspace=hypotheses_dir,
        readable_roots=[layout.outputs_root, layout.dash_src, layout.subject_dir],
        read_only=False,
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
