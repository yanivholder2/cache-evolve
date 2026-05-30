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


def _baseline_vector(champions: dict) -> Optional[dict]:
    for fv in champions.get("fitness_vectors", []):
        if fv.get("variant") == "baseline":
            return fv
    return None


def _render(cycles_data: list, champions: Optional[dict] = None) -> str:
    if not cycles_data:
        return "# Leaderboard\n\n_No cycles recorded yet._\n"

    pareto_front = set(champions["pareto_front"]) if champions else set()

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

    baseline_vector = _baseline_vector(champions) if champions else None
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
                _mark_variant(fv["variant"], row["cycle"], pareto_front, champions) for fv in fvs
            )
        else:
            variant_names = ", ".join(
                _mark_variant(name, row["cycle"], pareto_front, champions) for name in row["pareto_front"]
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

    if champions:
        lines.append("## Global best-so-far\n")
        lines.append("Pareto front: " + ", ".join(champions["pareto_front"]))
        lines.append("")
        for dim, best in champions.get("best_per_dimension", {}).items():
            lines.append(f"- {dim}: {best['summary']:.3g} held by {best['variant']}")
        lines.append("")

    return "\n".join(lines)


def _mark_variant(name: str, row_cycle: str, pareto_front: set, champions: Optional[dict]) -> str:
    if champions is None:
        return name
    global_id = f"cycle/{row_cycle}/{name}"
    if global_id not in pareto_front:
        return f"{name} (regressed)"
    return name


def regenerate(*, layout: OutputsLayout, output: Path | None = None) -> str:
    """Returns the rendered leaderboard markdown; writes it to `output` if given."""

    champions: Optional[dict] = None
    if layout.champions_json.is_file():
        try:
            champions = json.loads(read_utf8(layout.champions_json))
        except json.JSONDecodeError as e:
            raise LeaderboardError(f"malformed champions.json: {e}") from e

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

    content = _render(cycles_data, champions)
    if output is not None:
        write_utf8(output, content)
    return content
