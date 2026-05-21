"""Cohort scaffold stage: write cohort.md after hypotheses are allocated."""

from __future__ import annotations

from pathlib import Path


def write_cohort_md(cycle_dir: Path, hypothesis_files: list[Path]) -> None:
    lines = [
        f"# Cycle {cycle_dir.name} Cohort", "",
        f"Cohort size: {len(hypothesis_files)}", "",
        "## Hypotheses", "",
    ]
    for p in hypothesis_files:
        hid = p.stem
        lines.append(f"- [`{hid}`](hypotheses/{hid}.md)")
    (cycle_dir / "cohort.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
