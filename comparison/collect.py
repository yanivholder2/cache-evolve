"""Turn both arms' run directories into one table of candidates, re-score them
under identical conditions, and summarise.

Why re-score: in-loop fitness is measured while the framework is busy (cache-evolve
evaluates a cohort in parallel threads, OpenEvolve one program at a time), and
per-op-time is timing-sensitive. Each arm still *searches* with its own in-loop
score, as published; reported numbers come from a serial re-evaluation of every
gate-passing candidate through the same evaluator."""

from __future__ import annotations

import csv
import json
import statistics
from dataclasses import asdict, dataclass
from pathlib import Path

from comparison.common import (
    FAILED_SCORE, REPO_ROOT, evaluate_in_tempdir, load_configs, score_fitness_vector,
)

ARMS = ("evolve", "openevolve")


@dataclass
class Candidate:
    arm: str
    rep: int
    index: int  # 1-based position in the arm's submission order
    passed: bool
    stage: str
    loop_score: float  # score measured inside the search loop
    time: float | None  # epoch seconds the candidate finished evaluating
    source: str | None  # the evolved file, when recoverable
    extra_files: int = 0  # overlay files besides the evolved one (cache-evolve only)
    rescored: float | None = None  # serial re-evaluation, same evaluator for both arms


def collect_openevolve(run_dir: Path) -> list[Candidate]:
    log = run_dir / "evaluations.jsonl"
    if not log.is_file():
        return []
    rows = [json.loads(line) for line in log.read_text().splitlines() if line.strip()]
    rows = rows[1:]  # row 0 is OpenEvolve scoring the seed program
    out = []
    for position, row in enumerate(rows, start=1):
        source = run_dir / "candidates" / f"{row['index']:04d}.rs"
        out.append(Candidate(
            arm="openevolve", rep=0, index=position, passed=row["passed"], stage=row["stage"],
            loop_score=row["score"], time=row["time"],
            source=source.read_text() if source.is_file() else None,
        ))
    return out


def collect_evolve(outputs_dir: Path, subject: str, evolve_file: str) -> list[Candidate]:
    cfg = load_configs(REPO_ROOT, subject=subject)
    baseline_path = REPO_ROOT / "inputs" / "subjects" / subject / "baseline" / "eval" / "fitness-vector.json"
    baseline = json.loads(baseline_path.read_text())
    out = []
    variants = sorted((outputs_dir / "cycles").glob("*/variants/v*"))
    for position, variant_dir in enumerate(variants, start=1):
        attempts = sorted((variant_dir / "attempts").glob("*/overlay"))
        overlay = attempts[-1] if attempts else None
        evolved = overlay / evolve_file if overlay else None
        source = evolved.read_text() if evolved and evolved.is_file() else None
        extra = 0
        if overlay:
            extra = sum(1 for p in overlay.rglob("*") if p.is_file() and p != evolved)
        fitness = variant_dir / "eval" / "fitness-vector.json"
        if fitness.is_file():
            score = score_fitness_vector(cfg, json.loads(fitness.read_text()), baseline)
            out.append(Candidate("evolve", 0, position, True, "ok", score,
                                 fitness.stat().st_mtime, source, extra))
        else:
            rejection = variant_dir / "rejection.md"
            when = (rejection if rejection.is_file() else variant_dir).stat().st_mtime
            out.append(Candidate("evolve", 0, position, False, "rejected", FAILED_SCORE,
                                 when, source, extra))
    return out


def rescore(candidates: list[Candidate], *, subject: str, evolve_file: str) -> None:
    """Serially re-evaluate every gate-passing candidate whose source is a single file."""
    todo = [c for c in candidates if c.passed and c.source is not None and c.extra_files == 0]
    for n, cand in enumerate(todo, start=1):
        print(f"  rescore {cand.arm}/rep{cand.rep} #{cand.index} ({n}/{len(todo)})", flush=True)
        outcome = evaluate_in_tempdir(cand.source, subject=subject, evolve_file=evolve_file)
        cand.rescored = outcome.score


def reported(c: Candidate, rescored: bool) -> float | None:
    """The score a candidate is judged by; None if it is not measurable."""
    if not c.passed:
        return None
    return c.rescored if rescored else c.loop_score


def best_so_far(cands: list[Candidate], rescored: bool) -> list[float]:
    """Running best score, floored at the baseline (0): winners must beat it."""
    best, curve = 0.0, []
    for c in sorted(cands, key=lambda c: c.index):
        score = reported(c, rescored)
        if score is not None and score > best:
            best = score
        curve.append(best)
    return curve


def summarise(cands: list[Candidate], *, started: float | None, rescored: bool) -> dict:
    curve = best_so_far(cands, rescored)
    scores = [s for c in cands if (s := reported(c, rescored)) is not None]
    first_win = next((i + 1 for i, v in enumerate(curve) if v > 0), None)
    summary = {
        "candidates": len(cands),
        "passed_gates": sum(c.passed for c in cands),
        "pass_rate": (sum(c.passed for c in cands) / len(cands)) if cands else None,
        "beat_baseline": sum(1 for s in scores if s > 0),
        "best_score": curve[-1] if curve else 0.0,
        "median_passing_score": statistics.median(scores) if scores else None,
        "first_candidate_beating_baseline": first_win,
        "candidates_not_rescorable": sum(1 for c in cands if c.passed and c.rescored is None) if rescored else 0,
        "multi_file_candidates": sum(1 for c in cands if c.extra_files),
    }
    times = [c.time for c in cands if c.time]
    if started and times:
        summary["wall_minutes_to_last_candidate"] = round((max(times) - started) / 60, 1)
    return summary


def write_outputs(by_run: dict[tuple[str, int], list[Candidate]], starts: dict, out: Path,
                  *, rescored: bool) -> dict:
    rows = [c for cands in by_run.values() for c in cands]
    with (out / "candidates.csv").open("w", newline="") as f:
        fields = [k for k in asdict(rows[0]) if k != "source"] if rows else []
        writer = csv.DictWriter(f, fieldnames=fields)
        writer.writeheader()
        for c in rows:
            writer.writerow({k: v for k, v in asdict(c).items() if k != "source"})

    summary = {
        f"{arm}/rep{rep}": summarise(cands, started=starts.get((arm, rep)), rescored=rescored)
        for (arm, rep), cands in sorted(by_run.items())
    }
    (out / "summary.json").write_text(json.dumps(summary, indent=2))
    _plot(by_run, starts, out, rescored=rescored)
    return summary


def _plot(by_run, starts, out: Path, *, rescored: bool) -> None:
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    colors = {"evolve": "tab:blue", "openevolve": "tab:orange"}
    labels = {"evolve": "cache-evolve", "openevolve": "OpenEvolve"}
    fig, (left, right) = plt.subplots(1, 2, figsize=(11, 4))
    seen = set()
    for (arm, rep), cands in sorted(by_run.items()):
        ordered = sorted(cands, key=lambda c: c.index)
        curve = best_so_far(ordered, rescored)
        label = labels[arm] if arm not in seen else None
        seen.add(arm)
        left.step(range(1, len(curve) + 1), curve, where="post", color=colors[arm], label=label, alpha=0.85)
        start = starts.get((arm, rep))
        if start:
            xs = [(c.time - start) / 60 for c in ordered if c.time]
            if len(xs) == len(curve):
                right.step(xs, curve, where="post", color=colors[arm], alpha=0.85)
    left.set_xlabel("candidates evaluated")
    right.set_xlabel("wall-clock minutes")
    for ax in (left, right):
        ax.set_ylabel("best weighted improvement over baseline")
        ax.grid(alpha=0.3)
    left.legend()
    fig.suptitle("Best-so-far score" + (" (serial re-score)" if rescored else " (in-loop)"))
    fig.tight_layout()
    fig.savefig(out / "trajectory.png", dpi=150)
    plt.close(fig)
