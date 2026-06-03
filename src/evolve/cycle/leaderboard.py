"""Renders leaderboard.md by scanning the cycles directory for cycle-winners.json files."""

from __future__ import annotations

import json
import logging
from pathlib import Path
from typing import Optional

from evolve._io import read_utf8, write_utf8
from evolve.identifiers import CYCLE_ID_WIDTH, CycleId
from evolve.paths import OutputsLayout

log = logging.getLogger(__name__)


class LeaderboardError(Exception):
    pass


def _enumerate_cycle_ids(layout: OutputsLayout) -> list[CycleId]:
    cycles_dir = layout.cycles_dir
    if not cycles_dir.is_dir():
        return []
    ids: list[CycleId] = []
    for entry in cycles_dir.iterdir():
        if not entry.is_dir():
            continue
        if len(entry.name) != CYCLE_ID_WIDTH or not entry.name.isdigit():
            continue
        if not (entry / "cycle-winners.json").is_file():
            continue
        ids.append(CycleId.from_str(entry.name))
    ids.sort()
    return ids


def _load_winners(layout: OutputsLayout, cycle_id: CycleId) -> Optional[dict]:
    path = layout.cycle(cycle_id).cycle_winners_json
    if not path.exists():
        return None
    try:
        return json.loads(read_utf8(path))
    except json.JSONDecodeError as e:
        raise LeaderboardError(f"malformed cycle-winners.json for cycle {cycle_id}: {e}") from e


def _baseline_vector(cross_cycle_winners: dict) -> Optional[dict]:
    for fv in cross_cycle_winners.get("fitness_vectors", []):
        if fv.get("variant") == "baseline":
            return fv
    return None


def _render(cycles_data: list, cross_cycle_winners: Optional[dict] = None) -> str:
    if not cycles_data:
        return "# Leaderboard\n\n_No cycles recorded yet._\n"

    winners = set(cross_cycle_winners["winners"]) if cross_cycle_winners else set()

    all_dims: list = []
    for row in cycles_data:
        for fv in row["fitness_vectors"]:
            for d in fv.get("dimensions", []):
                if d["dim"] not in all_dims:
                    all_dims.append(d["dim"])

    if all_dims:
        header = "| Cycle | Variants | " + " | ".join(all_dims) + " |"
        sep = "| --- | --- | " + " | ".join(["---"] * len(all_dims)) + " |"
    else:
        header = "| Cycle | Variants |"
        sep = "| --- | --- |"

    lines = ["# Leaderboard\n", header, sep]

    baseline_vector = _baseline_vector(cross_cycle_winners) if cross_cycle_winners else None
    if baseline_vector is not None:
        if all_dims:
            scores = []
            for dim in all_dims:
                summary = next(
                    (d["summary"] for d in baseline_vector.get("dimensions", []) if d["dim"] == dim),
                    None,
                )
                scores.append(f"{summary:.3g}" if summary is not None else "—")
            lines.append("| baseline | baseline | " + " | ".join(scores) + " |")
        else:
            lines.append("| baseline | baseline |")

    for row in cycles_data:
        fvs = row["fitness_vectors"]
        if fvs:
            variant_names = ", ".join(
                _mark_variant(fv["variant"], row["cycle"], winners, cross_cycle_winners) for fv in fvs
            )
        else:
            variant_names = ", ".join(
                _mark_variant(name, row["cycle"], winners, cross_cycle_winners) for name in row["winners"]
            )

        if all_dims and fvs:
            scores = []
            for dim in all_dims:
                vals = []
                for fv in fvs:
                    for d in fv.get("dimensions", []):
                        if d["dim"] == dim:
                            vals.append(f"{d['summary']:.3g}")
                scores.append(", ".join(vals) if vals else "—")
            lines.append(f"| {row['cycle']} | {variant_names} | " + " | ".join(scores) + " |")
        else:
            lines.append(f"| {row['cycle']} | {variant_names} |")

    lines.append("")

    if cross_cycle_winners:
        lines.append("## Global best-so-far\n")
        lines.append("Pareto front: " + ", ".join(cross_cycle_winners["winners"]))
        lines.append("")
        for dim, best in cross_cycle_winners.get("best_per_dimension", {}).items():
            lines.append(f"- {dim}: {best['summary']:.3g} held by {best['variant']}")
        lines.append("")

    return "\n".join(lines)


def _mark_variant(name: str, row_cycle: str, winners: set, cross_cycle_winners: Optional[dict]) -> str:
    if cross_cycle_winners is None:
        return name
    global_id = f"cycle/{row_cycle}/{name}"
    if global_id not in winners:
        # Off the current global front means dominated by some cross-cycle winner, not
        # necessarily worse than its own parent or the baseline; "dominated"
        # avoids implying a regression that was never measured.
        return f"{name} (dominated)"
    return name


def regenerate(*, layout: OutputsLayout, output: Path | None = None) -> str:
    """Returns the rendered leaderboard markdown; writes it to `output` if given."""

    cross_cycle_winners: Optional[dict] = None
    if layout.cross_cycle_winners_json.is_file():
        try:
            cross_cycle_winners = json.loads(read_utf8(layout.cross_cycle_winners_json))
        except json.JSONDecodeError as e:
            raise LeaderboardError(f"malformed cross-cycle-winners.json: {e}") from e

    cycles_data = []
    for cycle_id in _enumerate_cycle_ids(layout):
        cycle_winners = _load_winners(layout, cycle_id)
        fvs = cycle_winners.get("fitness_vectors", []) if cycle_winners else []
        winners = cycle_winners.get("winners", []) if cycle_winners else []
        cycles_data.append({
            "cycle": str(cycle_id),
            "winners": winners,
            "fitness_vectors": fvs,
        })

    content = _render(cycles_data, cross_cycle_winners)
    if output is not None:
        write_utf8(output, content)
    return content
