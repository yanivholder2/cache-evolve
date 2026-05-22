"""Cohort scaffold stage: write cohort.md after hypotheses are allocated."""

from __future__ import annotations

from pathlib import Path

from src.paths import CycleLayout


def write_cohort_md(cycle_layout: CycleLayout, hypothesis_files: list[Path]) -> None:
    lines = [
        f"# Cycle {cycle_layout.num} Cohort", "",
        f"Cohort size: {len(hypothesis_files)}", "",
        "## Hypotheses", "",
    ]
    for p in hypothesis_files:
        hid = p.stem
        lines.append(f"- [`{hid}`](hypotheses/{hid}.md)")
    cycle_layout.cohort_md.write_text("\n".join(lines) + "\n", encoding="utf-8")
