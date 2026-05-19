#!/usr/bin/env python3
"""Cycle orchestrator — fans out a cohort of N Variants, runs real G1+G2+G3
gates and fitness plugins per variant, aggregates a Pareto front, seals the
cycle, and appends one ledger row.

Usage:
  [HYPOTHESES="h1\\nh2\\n..."] [COHORT_SIZE=N] [VARIANT_IMPLEMENTER=<cmd>] \\
  [EVOLVE_ROOT=<path>] [REPO_ROOT=<dash-crate-root>] [WORKER_ID=<id>] \\
  cycle-start.py

HYPOTHESES overrides COHORT_SIZE (one Variant per line).
COHORT_SIZE overrides the config.toml default.
VARIANT_IMPLEMENTER is invoked once per Variant in parallel as
  <cmd> <hypothesis-file> <impl-out-dir>
and defaults to scripts/variant-implementer.sh (copies the good fixture).
"""

from __future__ import annotations

import concurrent.futures
import json
import os
import re
import subprocess
import sys
from pathlib import Path

DEFAULT_COHORT_SIZE = 6


def die(msg: str, code: int = 1) -> "None":
    print(f"error: {msg}", file=sys.stderr)
    sys.exit(code)


def resolve_cohort_size(config_file: Path, env_value: str | None) -> tuple[int, str | None]:
    """Return (cohort_size, explicit_env_value).

    Precedence: env override > config.toml > built-in default.
    """
    explicit = env_value if env_value not in (None, "") else None
    if explicit is not None:
        return _validate_positive_int(explicit), explicit

    if config_file.is_file():
        for line in config_file.read_text().splitlines():
            m = re.match(r"\s*cohort_size\s*=\s*(\d+)", line)
            if m:
                return _validate_positive_int(m.group(1)), None
    return DEFAULT_COHORT_SIZE, None


def _validate_positive_int(value: str) -> int:
    if not re.fullmatch(r"\d+", value) or int(value) < 1:
        die(f"COHORT_SIZE must be a positive integer, got: {value}")
    return int(value)


def resolve_hypotheses(env_value: str | None, cohort_size: int, explicit_cohort: str | None) -> tuple[list[str], int]:
    """Return (hypotheses, effective_cohort_size).

    If HYPOTHESES is set: one Variant per non-empty line; warn if explicit
    COHORT_SIZE disagrees. Otherwise synthesize default stubs.
    """
    if env_value:
        hyps = [line for line in env_value.splitlines() if line]
        if explicit_cohort is not None and explicit_cohort != str(len(hyps)):
            print(
                f"note: HYPOTHESES has {len(hyps)} entries; COHORT_SIZE={explicit_cohort} ignored",
                file=sys.stderr,
            )
        return hyps, len(hyps)

    hyps = [
        f"Default hypothesis H{i:02d}: exploratory Variant from the walking-skeleton stub."
        for i in range(1, cohort_size + 1)
    ]
    return hyps, cohort_size


def next_cycle_num(cycles_dir: Path) -> str:
    cycles_dir.mkdir(parents=True, exist_ok=True)
    existing = [p.name for p in cycles_dir.iterdir() if re.fullmatch(r"\d{4}", p.name)]
    last = max((int(n) for n in existing), default=0)
    return f"{last + 1:04d}"


def scaffold_cohort(cycle_dir: Path, hypotheses: list[str]) -> None:
    """Write cohort.md plus hypotheses/HNN.md for every hypothesis."""
    hyp_dir = cycle_dir / "hypotheses"
    hyp_dir.mkdir(parents=True, exist_ok=True)

    lines = [f"# Cycle {cycle_dir.name} Cohort", "", f"Cohort size: {len(hypotheses)}", "", "## Hypotheses", ""]
    for i, text in enumerate(hypotheses, start=1):
        hid = f"H{i:02d}"
        lines.append(f"- [`{hid}`](hypotheses/{hid}.md)")
        (hyp_dir / f"{hid}.md").write_text(f"# Hypothesis {hid}\n\n{text}\n")
    (cycle_dir / "cohort.md").write_text("\n".join(lines) + "\n")


def fan_out_implementers(
    cycle_dir: Path,
    cohort_size: int,
    implementer: str,
) -> None:
    """Invoke VARIANT_IMPLEMENTER once per Variant in parallel. Aborts on any failure."""
    jobs = []
    for i in range(1, cohort_size + 1):
        hid = f"H{i:02d}"
        vid = f"v{i:03d}"
        variant_dir = cycle_dir / "variants" / vid
        (variant_dir / "impl").mkdir(parents=True, exist_ok=True)
        (variant_dir / "eval" / "fitness").mkdir(parents=True, exist_ok=True)
        jobs.append((vid, variant_dir, cycle_dir / "hypotheses" / f"{hid}.md"))

    def _run(job):
        vid, variant_dir, hyp_file = job
        log_path = variant_dir / "implementer.log"
        with open(log_path, "wb") as log:
            proc = subprocess.run(
                [implementer, str(hyp_file), str(variant_dir / "impl")],
                stdout=log,
                stderr=subprocess.STDOUT,
            )
        return vid, proc.returncode, log_path

    failed = False
    with concurrent.futures.ThreadPoolExecutor(max_workers=cohort_size) as pool:
        for vid, code, log_path in pool.map(_run, jobs):
            if code != 0:
                print(f"error: variant-implementer for {vid} failed (exit {code}):", file=sys.stderr)
                sys.stderr.write(log_path.read_text())
                failed = True
    if failed:
        sys.exit(1)


def run_gates_and_fitness(
    scripts_dir: Path,
    cycle_dir: Path,
    cycle_num: str,
    cohort_size: int,
    python: str,
) -> None:
    """Run G1+G2+G3 gates and three fitness plugins per variant, sequentially.

    Cargo serializes on the shared target dir, so sequential is intentional.
    """
    plugins = ("hit-rate", "per-op-time", "metadata-size")
    for i in range(1, cohort_size + 1):
        vid = f"v{i:03d}"
        variant_dir = cycle_dir / "variants" / vid
        impl_dir = variant_dir / "impl"

        gate = subprocess.run(
            ["bash", str(scripts_dir / "gate-check.sh"), str(impl_dir), cycle_num, vid]
        )
        if gate.returncode != 0:
            die(f"gate-check failed for {vid} in cycle {cycle_num}")

        fitness_dir = variant_dir / "eval" / "fitness"
        for plugin in plugins:
            out = fitness_dir / f"{plugin}.json"
            with open(out, "wb") as f:
                subprocess.run([python, str(scripts_dir / "fitness" / f"{plugin}.py")], stdout=f, check=True)
            if out.stat().st_size == 0:
                die(f"plugin {plugin} produced empty output for {vid} at {out}")
            try:
                json.loads(out.read_text())
            except json.JSONDecodeError:
                die(f"plugin {plugin} produced invalid JSON for {vid} at {out}")

        with open(variant_dir / "eval" / "fitness-vector.json", "wb") as f:
            subprocess.run(
                [python, str(scripts_dir / "fitness-aggregate.py"), vid, str(fitness_dir)],
                stdout=f,
                check=True,
            )


def collect_fitness_vectors(cycle_dir: Path, cohort_size: int) -> list[dict]:
    vectors = []
    for i in range(1, cohort_size + 1):
        vid = f"v{i:03d}"
        path = cycle_dir / "variants" / vid / "eval" / "fitness-vector.json"
        vectors.append(json.loads(path.read_text()))
    return vectors


def compute_pareto_front(scripts_dir: Path, python: str, fitness_vectors: list[dict]) -> list[str]:
    proc = subprocess.run(
        [python, str(scripts_dir / "pareto.py"), "-"],
        input=json.dumps(fitness_vectors),
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(proc.stdout)


def write_per_variant_insights(scripts_dir: Path, cycle_dir: Path, cohort_size: int, python: str) -> None:
    for i in range(1, cohort_size + 1):
        vid = f"v{i:03d}"
        eval_dir = cycle_dir / "variants" / vid / "eval"
        dim_summary = subprocess.run(
            [python, str(scripts_dir / "dim-summary.py"), str(eval_dir / "fitness-vector.json")],
            capture_output=True,
            text=True,
            check=True,
        ).stdout.rstrip("\n")
        (cycle_dir / "variants" / vid / "insight.md").write_text(
            f"# Variant {vid} Insight\n\nFitness: {dim_summary}\n"
        )


def main() -> None:
    script_dir = Path(__file__).resolve().parent
    evolve_root = Path(os.environ.get("EVOLVE_ROOT") or script_dir.parent).resolve()
    repo_root = Path(os.environ.get("REPO_ROOT") or evolve_root.parent).resolve()
    os.environ["EVOLVE_ROOT"] = str(evolve_root)
    os.environ["REPO_ROOT"] = str(repo_root)

    worker_id = os.environ.get("WORKER_ID") or "local"
    os.environ["WORKER_ID"] = worker_id

    implementer = os.environ.get("VARIANT_IMPLEMENTER") or str(script_dir / "variant-implementer.sh")

    cohort_size, explicit_cohort = resolve_cohort_size(
        evolve_root / "config.toml", os.environ.get("COHORT_SIZE")
    )
    hypotheses, cohort_size = resolve_hypotheses(
        os.environ.get("HYPOTHESES"), cohort_size, explicit_cohort
    )

    cycles_dir = evolve_root / "cycles"
    cycle_num = next_cycle_num(cycles_dir)
    cycle_dir = cycles_dir / cycle_num
    ledger = evolve_root / "ledger" / "cycles.jsonl"

    if cycle_dir.is_dir():
        die(f"cycle {cycle_num} already exists at {cycle_dir}")

    python_bin = str(evolve_root / ".venv" / "bin" / "python3")
    if not (os.path.isfile(python_bin) and os.access(python_bin, os.X_OK)):
        python_bin = "python3"

    scaffold_cohort(cycle_dir, hypotheses)
    fan_out_implementers(cycle_dir, cohort_size, implementer)
    run_gates_and_fitness(script_dir, cycle_dir, cycle_num, cohort_size, python_bin)

    fitness_vectors = collect_fitness_vectors(cycle_dir, cohort_size)
    pareto_front = compute_pareto_front(script_dir, python_bin, fitness_vectors)

    write_per_variant_insights(script_dir, cycle_dir, cohort_size, python_bin)

    (cycle_dir / "winners.json").write_text(
        json.dumps(
            {
                "cycle": cycle_num,
                "worker_id": worker_id,
                "pareto_front": pareto_front,
                "fitness_vectors": fitness_vectors,
            }
        )
    )

    pareto_str = json.dumps(pareto_front)
    (cycle_dir / "cycle-insight.md").write_text(
        f"# Cycle {cycle_num} Insight\n\n"
        f"Cohort size: {cohort_size}\n"
        f"Pareto front: {pareto_str}\n\n"
        f"Evaluated {cohort_size} variant(s) with real fitness plugins "
        f"(hit-rate, per-op-time, metadata-size).\n"
    )

    subprocess.run(["bash", str(script_dir / "seal.sh"), str(cycle_dir)], check=True)

    row = {
        "cycle": cycle_num,
        "worker_id": worker_id,
        "cycle_dir": f"evolve/cycles/{cycle_num}",
        "cohort_size": cohort_size,
        "pareto_front": pareto_front,
    }
    subprocess.run(
        [python_bin, str(script_dir / "ledger-append.py"), str(ledger)],
        input=json.dumps(row),
        text=True,
        check=True,
    )

    subprocess.run(
        [
            python_bin,
            str(script_dir / "leaderboard.py"),
            "--evolve-root",
            str(evolve_root),
            "--output",
            str(evolve_root / "leaderboard.md"),
        ],
        check=True,
    )

    print(f"cycle {cycle_num} complete — sealed at {cycle_dir} (cohort={cohort_size})")


if __name__ == "__main__":
    main()
