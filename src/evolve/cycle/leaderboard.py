"""Renders leaderboard.md by scanning the cycles directory for winners.json files."""

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
        if not (entry / "winners.json").is_file():
            continue
        ids.append(CycleId.from_str(entry.name))
    ids.sort()
    return ids


def _load_winners(layout: OutputsLayout, cycle_id: CycleId) -> Optional[dict]:
    path = layout.cycle(cycle_id).winners_json
    if not path.exists():
        return None
    try:
        return json.loads(read_utf8(path))
    except json.JSONDecodeError as e:
        raise LeaderboardError(f"malformed winners.json for cycle {cycle_id}: {e}") from e


def _render(cycles_data: list) -> str:
    if not cycles_data:
        return "# Leaderboard\n\n_No cycles recorded yet._\n"

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

    for row in cycles_data:
        fvs = row["fitness_vectors"]
        if fvs:
            variant_names = ", ".join(fv["variant"] for fv in fvs)
        else:
            variant_names = ", ".join(row["pareto_front"])

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
    return "\n".join(lines)


def regenerate(*, layout: OutputsLayout, output: Path | None = None) -> str:
    """Returns the rendered leaderboard markdown; writes it to `output` if given."""

    cycles_data = []
    for cycle_id in _enumerate_cycle_ids(layout):
        winners = _load_winners(layout, cycle_id)
        fvs = winners.get("fitness_vectors", []) if winners else []
        pareto_front = winners.get("pareto_front", []) if winners else []
        cycles_data.append({
            "cycle": str(cycle_id),
            "pareto_front": pareto_front,
            "fitness_vectors": fvs,
        })

    content = _render(cycles_data)
    if output is not None:
        write_utf8(output, content)
    return content
