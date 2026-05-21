"""Render leaderboard.md from the cycle ledger and per-cycle winners.json files.

Library: `regenerate(*, outputs_root, output=None) -> str`. Reads
outputs/ledger/cycles.jsonl and each cycle's winners.json, writes markdown to
`output` (if given) and returns the rendered content.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Optional


class LeaderboardError(Exception):
    pass


def _load_cycles(outputs_root: Path) -> list:
    ledger = outputs_root / "ledger" / "cycles.jsonl"
    if not ledger.exists():
        return []
    cycles = []
    for n, line in enumerate(ledger.read_text().splitlines(), start=1):
        line = line.strip()
        if not line:
            continue
        try:
            cycles.append(json.loads(line))
        except json.JSONDecodeError as e:
            raise LeaderboardError(f"malformed ledger line {n} in {ledger}: {e}") from e
    return cycles


def _load_winners(outputs_root: Path, cycle_id: str) -> Optional[dict]:
    path = outputs_root / "cycles" / cycle_id / "winners.json"
    if not path.exists():
        return None
    try:
        return json.loads(path.read_text())
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


def regenerate(*, outputs_root: Path, output: Path | None = None) -> str:
    """Build the markdown leaderboard from ledger + winners; optionally write
    to `output`. Returns the rendered string either way."""

    ledger_rows = _load_cycles(outputs_root)
    cycles_data = []
    for row in ledger_rows:
        cycle_id = row["cycle"]
        winners = _load_winners(outputs_root, cycle_id)
        fvs = winners.get("fitness_vectors", []) if winners else []
        pareto_front = winners.get("pareto_front", []) if winners else []
        cycles_data.append({
            "cycle": cycle_id,
            "pareto_front": pareto_front,
            "fitness_vectors": fvs,
        })

    content = _render(cycles_data)
    if output is not None:
        output.write_text(content)
    return content
