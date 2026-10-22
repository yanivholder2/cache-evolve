"""Run cache-evolve and OpenEvolve on the same subject with the same budget and
compare them.

    python -m comparison.run_comparison --subject lru_cache --budget 30 --cohort-size 3 \\
        --model claude-opus-4-8

Both arms get the same LLM model, the same number of candidate programs, the same
gates and the same scorer; see comparison/README.md for what is and is not equal.
Use --dry-run to print the commands, --collect-only to re-summarise a finished run.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from comparison.collect import (  # noqa: E402
    Candidate, collect_evolve, collect_openevolve, rescore, write_outputs,
)
from comparison.common import REPO_ROOT, default_evolve_file  # noqa: E402


def run(cmd: list[str], *, dry_run: bool, log: Path | None = None) -> None:
    print("$", " ".join(cmd), flush=True)
    if dry_run:
        return
    if log is None:
        subprocess.run(cmd, cwd=REPO_ROOT, check=True)
        return
    with log.open("w") as f:
        subprocess.run(cmd, cwd=REPO_ROOT, check=True, stdout=f, stderr=subprocess.STDOUT)


def run_evolve_arm(args, rep_dir: Path) -> Path:
    cycles = args.budget // args.cohort_size
    before = set((REPO_ROOT / "outputs").glob(f"{args.subject}_*"))
    cmd = [
        sys.executable, "-m", "evolve.cli",
        "--subject", args.subject, "--cycles", str(cycles), "--cohort-size", str(args.cohort_size),
        "--ai-agent-tool", "claude-sdk", "--model", args.model,
        "--authoring-scope", "full-autonomy", "--max-workers", str(args.evolve_max_workers),
    ]
    rep_dir.mkdir(parents=True, exist_ok=True)
    run(cmd, dry_run=args.dry_run, log=None if args.dry_run else rep_dir / "evolve.log")
    if args.dry_run:
        return rep_dir
    created = sorted(set((REPO_ROOT / "outputs").glob(f"{args.subject}_*")) - before)
    if not created:
        raise SystemExit("cache-evolve produced no outputs directory")
    (rep_dir / "outputs_path.txt").write_text(str(created[-1]))
    return created[-1]


def run_openevolve_arm(args, rep_dir: Path, rep: int) -> None:
    cmd = [
        sys.executable, "-m", "comparison.run_openevolve",
        "--subject", args.subject, "--iterations", str(args.budget), "--model", args.model,
        "--provider", args.openevolve_provider, "--seed", str(args.seed + rep), "--out", str(rep_dir),
    ]
    if args.evolve_file:
        cmd += ["--evolve-file", args.evolve_file]
    if args.openevolve_api_base:
        cmd += ["--api-base", args.openevolve_api_base]
    rep_dir.mkdir(parents=True, exist_ok=True)
    run(cmd, dry_run=args.dry_run, log=None if args.dry_run else rep_dir / "openevolve.log")


def load_run_start(rep_dir: Path) -> float | None:
    path = rep_dir / "started.json"
    return json.loads(path.read_text())["start"] if path.is_file() else None


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--subject", default="lru_cache")
    p.add_argument("--evolve-file", default=None, help="file OpenEvolve evolves (default: src/<subject>/mod.rs)")
    p.add_argument("--budget", type=int, default=30, help="candidate programs per arm")
    p.add_argument("--cohort-size", type=int, default=3, help="cache-evolve variants per cycle")
    p.add_argument("--model", default="claude-opus-4-8", help="same model for both arms")
    p.add_argument("--repeats", type=int, default=1)
    p.add_argument("--seed", type=int, default=42, help="OpenEvolve seed (rep k uses seed+k)")
    p.add_argument("--arms", nargs="+", choices=["evolve", "openevolve"], default=["evolve", "openevolve"])
    p.add_argument("--evolve-max-workers", type=int, default=1,
                   help="cache-evolve variant fan-out; 1 keeps timing measurements quiet")
    p.add_argument("--openevolve-provider", choices=["claude_code", "openai"], default="claude_code")
    p.add_argument("--openevolve-api-base", default=None)
    p.add_argument("--out", type=Path, default=None, help="default: comparison_runs/<subject>-<timestamp>")
    p.add_argument("--skip-rescore", action="store_true")
    p.add_argument("--collect-only", action="store_true", help="skip running; summarise --out")
    p.add_argument("--dry-run", action="store_true")
    args = p.parse_args(argv)

    if args.budget % args.cohort_size:
        p.error("--budget must be a multiple of --cohort-size (cache-evolve runs whole cycles)")
    evolve_file = args.evolve_file or default_evolve_file(args.subject)
    out = args.out or REPO_ROOT / "comparison_runs" / f"{args.subject}-{time.strftime('%Y%m%d-%H%M%S')}"
    if args.collect_only and not out.is_dir():
        p.error(f"--collect-only needs an existing --out (got {out})")
    out.mkdir(parents=True, exist_ok=True)
    (out / "config.json").write_text(json.dumps({**vars(args), "out": str(out), "evolve_file": evolve_file}, indent=2, default=str))

    if not args.collect_only:
        for rep in range(args.repeats):
            for arm in args.arms:
                rep_dir = out / arm / f"rep{rep}"
                rep_dir.mkdir(parents=True, exist_ok=True)
                if not args.dry_run:
                    (rep_dir / "started.json").write_text(json.dumps({"start": time.time()}))
                if arm == "evolve":
                    run_evolve_arm(args, rep_dir)
                else:
                    run_openevolve_arm(args, rep_dir, rep)
    if args.dry_run:
        return 0

    by_run: dict[tuple[str, int], list[Candidate]] = {}
    starts: dict[tuple[str, int], float | None] = {}
    for arm_dir in sorted(p for p in out.iterdir() if p.is_dir() and p.name in ("evolve", "openevolve")):
        for rep_dir in sorted(arm_dir.glob("rep*")):
            rep = int(rep_dir.name.removeprefix("rep"))
            if arm_dir.name == "evolve":
                path = rep_dir / "outputs_path.txt"
                cands = collect_evolve(Path(path.read_text()), args.subject, evolve_file) if path.is_file() else []
            else:
                cands = collect_openevolve(rep_dir)
            for c in cands:
                c.rep = rep
            by_run[(arm_dir.name, rep)] = cands
            starts[(arm_dir.name, rep)] = load_run_start(rep_dir)

    rescored = not args.skip_rescore
    if rescored:
        print("re-scoring gate-passing candidates serially ...", flush=True)
        for cands in by_run.values():
            rescore(cands, subject=args.subject, evolve_file=evolve_file)
    summary = write_outputs(by_run, starts, out, rescored=rescored)
    print(json.dumps(summary, indent=2))
    print(f"wrote {out}/summary.json, candidates.csv, trajectory.png")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
