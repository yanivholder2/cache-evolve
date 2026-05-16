#!/usr/bin/env python3
"""Render leaderboard.md from the cycle ledger and per-cycle winners.json files.

Usage: leaderboard.py --evolve-root <path> [--output <path>]

Reads evolve/ledger/cycles.jsonl and each cycle's winners.json,
then writes (or prints) a markdown leaderboard.
"""

import argparse
import json
import sys
from pathlib import Path
from typing import Optional


def _load_cycles(evolve_root: Path) -> list:
    ledger = evolve_root / "ledger" / "cycles.jsonl"
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
            print(f"error: malformed ledger line {n} in {ledger}: {e}", file=sys.stderr)
            sys.exit(1)
    return cycles


def _load_winners(evolve_root: Path, cycle_id: str) -> Optional[dict]:
    path = evolve_root / "cycles" / cycle_id / "winners.json"
    if not path.exists():
        return None
    try:
        return json.loads(path.read_text())
    except json.JSONDecodeError as e:
        print(f"error: malformed winners.json for cycle {cycle_id}: {e}", file=sys.stderr)
        sys.exit(1)


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


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--evolve-root", required=True, type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    ledger_rows = _load_cycles(args.evolve_root)

    cycles_data = []
    for row in ledger_rows:
        cycle_id = row["cycle"]
        winners = _load_winners(args.evolve_root, cycle_id)
        fvs = winners.get("fitness_vectors", []) if winners else []
        pareto_front = winners.get("pareto_front", []) if winners else []
        cycles_data.append({
            "cycle": cycle_id,
            "pareto_front": pareto_front,
            "fitness_vectors": fvs,
        })

    content = _render(cycles_data)

    if args.output:
        args.output.write_text(content)
    else:
        sys.stdout.write(content)


if __name__ == "__main__":
    main()
