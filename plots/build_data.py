#!/usr/bin/env python
"""Aggregate real run outputs under outputs/ into plots/data.json.

Emits one row per variant per cycle, plus a baseline row when the run
recorded one. Each canonical fitness dimension absent from a variant's
vector is written as -1.

Run-setup fields are read from the config.toml snapshot the runner copies
into each run folder at start. Runs predating that snapshot fall back to
the authoritative cycle count and ledger cohort size, with the rest left
as "unknown" / -1.

Idempotent: regenerates data.json from scratch on every invocation.
Stdlib only (tomllib needs Python 3.11+). Run with the project venv:

    .venv/bin/python plots/build_data.py
"""

from __future__ import annotations

import argparse
import json
import tomllib
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
OUTPUTS_DIR = REPO_ROOT / "outputs"
PLOTS_DIR = REPO_ROOT / "plots"
DATA_PATH = PLOTS_DIR / "data.json"

UNKNOWN = "unknown"

# Every fitness dimension the framework currently emits. A variant whose
# vector omits one of these gets MISSING for that dimension.
CANONICAL_DIMS = ("hit-rate", "metadata-size", "per-op-time")
MISSING = -1


def subject_of(run_name: str) -> str:
    """Run dir is "<subject>_<YYYY-MM-DD>_<HH-MM-SS-mmm>"; strip the timestamp."""
    return run_name.rsplit("_", 2)[0]


def dim_summaries(vector: dict) -> dict:
    return {dim["dim"]: dim.get("summary", MISSING) for dim in vector.get("dimensions", [])}


def make_row(summaries, *, subject, cycle, variant, is_baseline, is_winner):
    row = {
        "subject-name": subject,
        "cycle": cycle,
        "variant": variant,
        "is-baseline": is_baseline,
        "is-winner": is_winner,
    }
    for dim in CANONICAL_DIMS:
        row[dim] = summaries.get(dim, MISSING)
    return row


def winners_by_cycle(run_dir: Path) -> dict:
    """Map cycle number into the set of winning variant ids for that cycle.

    Ledger vintages name the field either "winners" or "pareto_front"; both
    hold bare variant ids (e.g. ["v001"]).
    """
    winners = {}
    for line in (run_dir / "cycles-ledger.jsonl").read_text().splitlines():
        if not line.strip():
            continue
        record = json.loads(line)
        ids = record.get("winners", record.get("pareto_front", []))
        winners[int(record["cycle"])] = set(ids)
    return winners


def ledger_cohort_size(run_dir: Path) -> int:
    for line in (run_dir / "cycles-ledger.jsonl").read_text().splitlines():
        if line.strip():
            return int(json.loads(line)["cohort_size"])
    return MISSING


def run_setup(run_dir: Path, subject: str, cycle_count: int) -> dict:
    """Build the run's setup block.

    Cycle count and ledger cohort size are authoritative run artifacts and
    always win. Everything else comes from the config.toml snapshot the runner
    copies into the run folder; runs predating that snapshot get sentinels.
    """
    setup = {
        "run": run_dir.name,
        "subject-name": subject,
        "total-cycles": cycle_count,
        "cohort-size": ledger_cohort_size(run_dir),
        "ai-agent-tool": UNKNOWN,
        "implementation-attempts": MISSING,
        "hypothesis-attempts": MISSING,
        "authoring-scope": UNKNOWN,
        "fitness-dimensions": [],
    }
    config_path = run_dir / "config.toml"
    if config_path.exists():
        config = tomllib.loads(config_path.read_text())
        framework = config.get("framework", {})
        fitness = config.get("fitness", {})
        setup["ai-agent-tool"] = framework.get("ai_agent_tool", UNKNOWN)
        setup["implementation-attempts"] = framework.get("implementation_attempts", MISSING)
        setup["hypothesis-attempts"] = framework.get("hypothesis_attempts", MISSING)
        setup["authoring-scope"] = framework.get("authoring_scope", UNKNOWN)
        setup["fitness-dimensions"] = fitness.get("dimensions", [])
    return setup


def baseline_row(run_dir: Path, subject: str):
    """Pull the baseline vector from champions.json, if the run recorded one."""
    champions = run_dir / "champions.json"
    if not champions.exists():
        return None
    data = json.loads(champions.read_text())
    for vector in data.get("fitness_vectors", []):
        if vector.get("variant") == "baseline":
            return make_row(
                dim_summaries(vector),
                subject=subject,
                cycle=0,
                variant="baseline",
                is_baseline=True,
                is_winner=False,
            )
    return None


def build_run(run_dir: Path):
    """Return the {setup, variants} block for one run, or None if incomplete."""
    if not (run_dir / "cycles-ledger.jsonl").exists():
        return None  # run never reached a sealed cycle
    subject = subject_of(run_dir.name)
    winners = winners_by_cycle(run_dir)

    variants = []
    baseline = baseline_row(run_dir, subject)
    if baseline is not None:
        variants.append(baseline)

    cycle_dirs = sorted((run_dir / "cycles").glob("[0-9]" * 4))
    for cycle_dir in cycle_dirs:
        cycle = int(cycle_dir.name)
        cycle_winners = winners.get(cycle, set())
        for variant_dir in sorted((cycle_dir / "variants").glob("v*")):
            vector_path = variant_dir / "eval" / "fitness-vector.json"
            if not vector_path.exists():
                continue  # variant produced no vector (e.g. build or gate failure)
            summaries = dim_summaries(json.loads(vector_path.read_text()))
            variants.append(
                make_row(
                    summaries,
                    subject=subject,
                    cycle=cycle,
                    variant=variant_dir.name,
                    is_baseline=False,
                    is_winner=variant_dir.name in cycle_winners,
                )
            )

    setup = run_setup(run_dir, subject, len(cycle_dirs))
    return {"setup": setup, "variants": variants}


def resolve_run_dirs(selectors: list[str]) -> list[Path]:
    """Map CLI selectors to run dirs. A selector may be a run dir name (looked
    up under outputs/) or a path to a run dir. Empty selectors means every run
    under outputs/."""
    if not selectors:
        return sorted(p for p in OUTPUTS_DIR.iterdir() if p.is_dir())
    run_dirs = []
    for selector in selectors:
        path = Path(selector)
        run_dir = path if path.is_absolute() or path.exists() else OUTPUTS_DIR / selector
        if not run_dir.is_dir():
            raise SystemExit(f"run dir not found: {run_dir}")
        run_dirs.append(run_dir)
    return run_dirs


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "runs",
        nargs="*",
        help="run dir name(s) or path(s) to aggregate; default: every run under outputs/",
    )
    args = parser.parse_args()

    runs = []
    for run_dir in resolve_run_dirs(args.runs):
        built = build_run(run_dir)
        if built is not None:
            runs.append(built)
    DATA_PATH.write_text(json.dumps({"runs": runs}, indent=2) + "\n")
    variant_count = sum(len(run["variants"]) for run in runs)
    print(f"Wrote {DATA_PATH} with {len(runs)} runs and {variant_count} variant rows.")


if __name__ == "__main__":
    main()
