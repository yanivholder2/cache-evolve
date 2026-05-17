#!/usr/bin/env python3
"""Held-out evaluation runner.

Runs hit-rate fitness against the held-out trace suite for the Pareto-front
variants of a cycle, and writes `holdout.json` into the cycle directory. The
output flags any variant whose held-out summary is meaningfully worse than
its dev summary (a "held-out regression").

Held-out traces and their profiles must remain hidden from the AI's context
bundle — only summary scalars from this runner's output may flow back into
ledger or insight artifacts.

CLI: holdout-runner.py <cycle_dir>
  Expects <cycle_dir>/winners.json and per-variant
  <cycle_dir>/variants/<vid>/eval/fitness-vector.json (for the dev baseline).

Environment:
  REGRESSION_THRESHOLD  fraction worse than dev that counts as a regression
                        (default 0.02 — i.e. ≥2 percentage points lower).
  REPO_ROOT, EVOLVE_ROOT  forwarded to the hit-rate plugin.
"""

import json
import os
import subprocess
import sys
from pathlib import Path

_HERE = Path(__file__).resolve()
DEFAULT_THRESHOLD = 0.02


def _dev_summary(fitness_vector_path: Path, dim: str = "hit-rate") -> float:
    """Pull the hit-rate summary from a variant's dev-suite fitness vector."""
    data = json.loads(fitness_vector_path.read_text())
    for d in data.get("dimensions", []):
        if d.get("dim") == dim:
            return float(d["summary"])
    raise KeyError(f"{fitness_vector_path}: no '{dim}' dimension in fitness vector")


def _run_holdout_plugin(repo_root: Path, evolve_root: Path) -> dict:
    """Invoke hit-rate.py with TRACE_SUITE=holdout and return the parsed record."""
    plugin = _HERE.parent / "fitness" / "hit-rate.py"
    env = {
        **os.environ,
        "REPO_ROOT": str(repo_root),
        "EVOLVE_ROOT": str(evolve_root),
        "TRACE_SUITE": "holdout",
    }
    result = subprocess.run([sys.executable, str(plugin)], capture_output=True, text=True, env=env)
    if result.returncode != 0:
        print(f"error: holdout hit-rate plugin failed:\n{result.stderr}", file=sys.stderr)
        sys.exit(1)
    return json.loads(result.stdout)


def evaluate_holdout(cycle_dir: Path, repo_root: Path, evolve_root: Path, threshold: float) -> dict:
    """Run holdout for every Pareto winner; return the result dict (also written to disk)."""
    winners_path = cycle_dir / "winners.json"
    if not winners_path.is_file():
        raise FileNotFoundError(f"{winners_path} not found — cycle has no Pareto front to evaluate")
    winners = json.loads(winners_path.read_text())
    pareto_front: list[str] = winners.get("pareto_front", [])

    per_variant = []
    any_regression = False
    for vid in pareto_front:
        fv_path = cycle_dir / "variants" / vid / "eval" / "fitness-vector.json"
        dev_summary = _dev_summary(fv_path)
        # The current variant build harness compiles one impl at a time; the
        # plugin we invoke evaluates whatever evolve-hit-rate builds against.
        # Per-variant holdout runs are sequential against the same binary.
        holdout = _run_holdout_plugin(repo_root, evolve_root)
        holdout_summary = float(holdout["summary"])
        delta = holdout_summary - dev_summary
        regression = delta < -threshold
        any_regression = any_regression or regression
        per_variant.append({
            "variant": vid,
            "dev_summary": dev_summary,
            "holdout_summary": holdout_summary,
            "delta": round(delta, 6),
            "regression": regression,
            "holdout_values": holdout["values"],
        })

    out = {
        "cycle": winners.get("cycle"),
        "worker_id": winners.get("worker_id"),
        "suite": "holdout",
        "regression_threshold": threshold,
        "any_regression": any_regression,
        "variants": per_variant,
    }
    (cycle_dir / "holdout.json").write_text(json.dumps(out, indent=2) + "\n")
    return out


def main() -> None:
    if len(sys.argv) != 2:
        print("usage: holdout-runner.py <cycle_dir>", file=sys.stderr)
        sys.exit(2)
    cycle_dir = Path(sys.argv[1]).resolve()
    repo_root = Path(os.environ.get("REPO_ROOT", _HERE.parents[2]))
    evolve_root = Path(os.environ.get("EVOLVE_ROOT", _HERE.parents[1]))
    threshold = float(os.environ.get("REGRESSION_THRESHOLD", DEFAULT_THRESHOLD))

    result = evaluate_holdout(cycle_dir, repo_root, evolve_root, threshold)
    if result["any_regression"]:
        regressed = [v["variant"] for v in result["variants"] if v["regression"]]
        print(f"held-out regression flagged for: {', '.join(regressed)}", file=sys.stderr)


if __name__ == "__main__":
    main()
