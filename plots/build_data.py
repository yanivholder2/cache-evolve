#!/usr/bin/env python
"""Aggregate real run outputs under outputs/paper_outputs/ into per-group JSON.

Runs are grouped under outputs/paper_outputs/<group>/<subject>/<run>/, where
<group> is the fitness configuration the runs share (e.g. only_hit_rate,
full_fitness_vector). Each group becomes its own plots/paper-plots/data_<group>.json
so hit-rate-only and full-fitness-vector runs stay in separate datasets.

Emits one row per variant per cycle, plus a baseline row when the run
recorded one. Each canonical fitness dimension absent from a variant's
vector is written as -1.

Run-setup fields are read from the config.toml snapshot the runner copies
into each run folder at start. Runs predating that snapshot fall back to
the authoritative cycle count and ledger cohort size, with the rest left
as "unknown" / -1.

Idempotent: regenerates the data files from scratch on every invocation.
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
PAPER_OUTPUTS_DIR = OUTPUTS_DIR / "paper_outputs"
PLOTS_DIR = REPO_ROOT / "plots"
PAPER_PLOTS_DIR = PLOTS_DIR / "paper-plots"

UNKNOWN = "unknown"

# Every fitness dimension the framework currently emits. A variant whose
# vector omits one of these gets MISSING for that dimension.
CANONICAL_DIMS = ("hit-rate", "metadata-size", "per-op-time")
MISSING = -1


def dim_summaries(vector: dict) -> dict:
    return {dim["dim"]: dim.get("summary", MISSING) for dim in vector.get("dimensions", [])}


# Dimensions the baseline-vs-winner diff re-measures and backfills onto rows
# whose own vector never recorded them (hit-rate-only runs). hit-rate already
# comes from the run's records, so the diff never overrides it on a real row.
BACKFILL_DIMS = ("metadata-size", "per-op-time")


def try_number(text: str):
    """Parse a table cell into int/float, or None when it is not numeric (e.g.
    the "—" the leaderboard writes for an absent score)."""
    text = text.strip()
    try:
        return float(text) if "." in text else int(text)
    except ValueError:
        return None


def diff_variant_key(variant_cell: str):
    """Map a diff-file Variant cell to a (cycle, variant) row key, or None.

    "baseline" pairs with cycle 0; winners read as "cycle/<NNNN>/<vID>" and the
    same variant id can recur across cycles with different re-measured numbers,
    so the cycle is part of the key.
    """
    if variant_cell == "baseline":
        return (0, "baseline")
    parts = variant_cell.split("/")
    if len(parts) == 3 and parts[0] == "cycle":
        return (int(parts[1]), parts[2])
    return None


def fitness_diff_rows(run_dir: Path) -> dict:
    """Parse baseline-vs-winner-fitness-diff.md into {(cycle, variant): dims}.

    Hit-rate-only runs carry this hand-authored file with metadata-size and
    per-op-time re-measured together (on one machine) for the baseline and the
    net-beating winners; it backfills dims the run itself never recorded. An
    absent file (e.g. full-fitness-vector runs) means no rows.
    """
    diff_path = run_dir / "baseline-vs-winner-fitness-diff.md"
    if not diff_path.exists():
        return {}
    rows = {}
    header = None
    for line in diff_path.read_text().splitlines():
        if not line.strip().startswith("|"):
            continue
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if header is None:
            if "Variant" in cells:
                header = cells
            continue
        if all(set(cell) <= {"-"} for cell in cells):
            continue  # markdown separator row
        record = dict(zip(header, cells))
        key = diff_variant_key(record.get("Variant", ""))
        if key is None:
            continue
        dims = {}
        for dim in (*CANONICAL_DIMS, "weighted-score"):
            column = "Weighted score" if dim == "weighted-score" else dim
            value = try_number(record.get(column, ""))
            if value is not None:
                dims[dim] = value
        rows[key] = dims
    return rows


def make_row(
    summaries,
    *,
    subject,
    cycle,
    variant,
    is_baseline,
    is_winner,
    count_attempts=MISSING,
    variant_seconds_to_generate=MISSING,
    total_seconds_to_generate=MISSING,
    cost_usd=MISSING,
):
    row = {
        "subject-name": subject,
        "cycle": cycle,
        "variant": variant,
        "is-baseline": is_baseline,
        "is-winner": is_winner,
        "count-attempts": count_attempts,
        "variant-seconds-to-generate": variant_seconds_to_generate,
        "total-seconds-to-generate": total_seconds_to_generate,
        "cost-usd": cost_usd,
    }
    for dim in CANONICAL_DIMS:
        row[dim] = summaries.get(dim, MISSING)
    row["weighted-score"] = MISSING
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


def backfill(row: dict, dims: dict) -> dict:
    """Fill a row's re-measured dims from the diff, leaving hit-rate untouched."""
    for dim in BACKFILL_DIMS:
        if dim in dims:
            row[dim] = dims[dim]
    return row


def leaderboard_records(run_dir: Path):
    """Yield each leaderboard.md data row as a header-mapped cell dict."""
    path = run_dir / "leaderboard.md"
    if not path.exists():
        return
    header = None
    for line in path.read_text().splitlines():
        if not line.strip().startswith("|"):
            continue
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if header is None:
            if "Cycle" in cells:
                header = cells
            continue
        if all(set(cell) <= {"-"} for cell in cells):
            continue  # markdown separator row
        yield dict(zip(header, cells))


def leaderboard_baseline_dims(run_dir: Path) -> dict:
    """Baseline fitness dims from the leaderboard's baseline row.

    Every run writes leaderboard.md with a "baseline" row, so this is the one
    baseline source common to all runs. Hit-rate-only leaderboards carry just
    hit-rate; full-fitness ones also carry metadata-size and per-op-time.
    """
    for record in leaderboard_records(run_dir):
        if record.get("Cycle") != "baseline":
            continue
        dims = {}
        for dim in CANONICAL_DIMS:
            value = try_number(record.get(dim, ""))
            if value is not None:
                dims[dim] = value
        return dims
    return {}


def leaderboard_scores(run_dir: Path) -> dict:
    """Map (cycle, variant) into the weighted score from the leaderboard.

    A cycle row's Variants and Score cells are comma-aligned lists; winners
    carry a "(winner)" suffix in the Variants cell. Hit-rate-only runs leave
    Score as "—" and so contribute nothing.
    """
    scores = {}
    for record in leaderboard_records(run_dir):
        cycle_cell = record.get("Cycle", "")
        if not cycle_cell.isdigit():
            continue  # baseline row carries no per-variant score
        cycle = int(cycle_cell)
        variants = [part.split()[0] for part in record.get("Variants", "").split(",")]
        cells = record.get("Score", "").split(",")
        for variant, cell in zip(variants, cells):
            value = try_number(cell)
            if value is not None:
                scores[(cycle, variant)] = value
    return scores


def collect_scores(run_dir: Path, diff_rows: dict) -> dict:
    """Weighted scores keyed by (cycle, variant), from the leaderboard and the
    diff. Where any score exists, the baseline anchors the scale at 0."""
    scores = leaderboard_scores(run_dir)
    for key, dims in diff_rows.items():
        if "weighted-score" in dims:
            scores[key] = dims["weighted-score"]
    if scores:
        scores.setdefault((0, "baseline"), 0)
    return scores


def baseline_row(run_dir: Path, subject: str, diff_rows: dict):
    """Build the baseline row, or None if no source records a baseline.

    The leaderboard's baseline row is the universal source (every run has it).
    The diff's re-measured baseline is overlaid on top where present, so dash
    hit-rate-only runs get the same machine's metadata-size and per-op-time
    their winners were backfilled with.
    """
    dims = {**leaderboard_baseline_dims(run_dir), **diff_rows.get((0, "baseline"), {})}
    if not dims:
        return None
    return make_row(
        dims,
        subject=subject,
        cycle=0,
        variant="baseline",
        is_baseline=True,
        is_winner=False,
    )


def extract_agent_stats(log_path: Path):
    duration = 0.0
    cost = 0.0
    if log_path.exists():
        for line in reversed(log_path.read_text(encoding="utf-8").splitlines()[-20:]):
            line = line.strip()
            if line.startswith("- duration:"):
                try:
                    duration = float(line.split(":", 1)[1].replace("s", "").strip())
                except ValueError:
                    pass
            elif line.startswith("- cost_usd:"):
                try:
                    cost = float(line.split(":", 1)[1].strip())
                except ValueError:
                    pass
    return duration, cost


def build_run(run_dir: Path, subject: str):
    """Return the {setup, variants} block for one run, or None if incomplete.

    Subject comes from the run's parent folder; the run dir name embeds the
    agent (e.g. "lru_cache_claude-opus-4-8_<date>_<time>") and is not safely
    parseable for subjects whose names contain underscores.
    """
    if not (run_dir / "cycles-ledger.jsonl").exists():
        return None  # run never reached a sealed cycle
    winners = winners_by_cycle(run_dir)
    diff_rows = fitness_diff_rows(run_dir)
    scores = collect_scores(run_dir, diff_rows)

    variants = []
    baseline = baseline_row(run_dir, subject, diff_rows)
    if baseline is not None:
        baseline["weighted-score"] = scores.get((0, "baseline"), MISSING)
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
            
            count_attempts = 0
            variant_seconds = 0.0
            total_seconds = 0.0
            cost = 0.0
            
            attempts_dir = variant_dir / "attempts"
            if attempts_dir.exists():
                for attempt_dir in attempts_dir.iterdir():
                    if attempt_dir.is_dir():
                        count_attempts += 1
                        d, c = extract_agent_stats(attempt_dir / "variant-implementer.log.md")
                        variant_seconds += d
                        total_seconds += d
                        cost += c
            else:
                count_attempts = 1
                
            ins_d, ins_c = extract_agent_stats(variant_dir / "variant-insight-writer.log.md")
            total_seconds += ins_d
            cost += ins_c
            
            row = backfill(
                make_row(
                    summaries,
                    subject=subject,
                    cycle=cycle,
                    variant=variant_dir.name,
                    is_baseline=False,
                    is_winner=variant_dir.name in cycle_winners,
                    count_attempts=count_attempts,
                    variant_seconds_to_generate=round(variant_seconds, 2) if variant_seconds else MISSING,
                    total_seconds_to_generate=round(total_seconds, 2) if total_seconds else MISSING,
                    cost_usd=round(cost, 6) if cost else MISSING,
                ),
                diff_rows.get((cycle, variant_dir.name), {}),
            )
            row["weighted-score"] = scores.get((cycle, variant_dir.name), MISSING)
            variants.append(row)

    setup = run_setup(run_dir, subject, len(cycle_dirs))
    return {"setup": setup, "variants": variants}


def group_run_dirs(group_dir: Path) -> list[tuple[Path, str]]:
    """List (run_dir, subject) pairs under a group's <subject>/<run> folders."""
    pairs = []
    for subject_dir in sorted(p for p in group_dir.iterdir() if p.is_dir()):
        for run_dir in sorted(p for p in subject_dir.iterdir() if p.is_dir()):
            pairs.append((run_dir, subject_dir.name))
    return pairs


def resolve_groups(selectors: list[str]) -> list[Path]:
    """Map CLI selectors to group dirs under outputs/paper_outputs/. A selector
    is a group folder name. Empty selectors means every group."""
    if not selectors:
        return sorted(p for p in PAPER_OUTPUTS_DIR.iterdir() if p.is_dir())
    group_dirs = []
    for selector in selectors:
        group_dir = PAPER_OUTPUTS_DIR / selector
        if not group_dir.is_dir():
            raise SystemExit(f"group dir not found: {group_dir}")
        group_dirs.append(group_dir)
    return group_dirs


def build_group(group_dir: Path) -> Path:
    """Aggregate one group into plots/paper-plots/data_<group>.json; return its path."""
    runs = []
    for run_dir, subject in group_run_dirs(group_dir):
        built = build_run(run_dir, subject)
        if built is not None:
            runs.append(built)
    PAPER_PLOTS_DIR.mkdir(parents=True, exist_ok=True)
    data_path = PAPER_PLOTS_DIR / f"data_{group_dir.name}.json"
    data_path.write_text(json.dumps({"runs": runs}, indent=2) + "\n")
    variant_count = sum(len(run["variants"]) for run in runs)
    print(f"Wrote {data_path} with {len(runs)} runs and {variant_count} variant rows.")
    return data_path


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "groups",
        nargs="*",
        help="group folder name(s) under outputs/paper_outputs/ to aggregate; "
        "default: every group",
    )
    args = parser.parse_args()

    for group_dir in resolve_groups(args.groups):
        build_group(group_dir)


if __name__ == "__main__":
    main()
