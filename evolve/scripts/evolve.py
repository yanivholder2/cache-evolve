#!/usr/bin/env python3
"""Evolve framework entry — runs `EVOLVE_CYCLES` (default 1) cycles. Each
cycle fans out a cohort of N Variants, runs real G1+G2+G3 gates and fitness
plugins per variant, aggregates a Pareto front, seals the cycle, and appends
one ledger row.

Usage:
  [EVOLVE_CYCLES=N] [HYPOTHESES="h1\\nh2\\n..."] [COHORT_SIZE=N] \\
  [VARIANT_IMPLEMENTER=<cmd>] [EVOLVE_LLM_CMD=<adapter>] \\
  [EVOLVE_ROOT=<path>] [REPO_ROOT=<dash-crate-root>] [WORKER_ID=<id>] \\
  evolve.py

HYPOTHESES (one Variant per line) overrides COHORT_SIZE and bypasses the
hypothesis-writer agent. Without HYPOTHESES, the wrapper invokes the LLM
adapter (ADR-0005) and verifies it wrote N H<NN>.md files.

VARIANT_IMPLEMENTER is invoked once per Variant in parallel as
  <cmd> <hypothesis-file> <impl-out-dir>
and defaults to scripts/variant-implementer.sh (copies the good fixture).
"""

from __future__ import annotations

import concurrent.futures
import json
import logging
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import NoReturn

DEFAULT_COHORT_SIZE = 6

log = logging.getLogger("evolve")


def die(msg: str) -> NoReturn:
    log.error(msg)
    print(f"error: {msg}", file=sys.stderr)
    sys.exit(1)


def variant_dir(cycle_dir: Path, i: int) -> Path:
    return cycle_dir / "variants" / f"v{i:03d}"


def setup_logging(log_file: Path | None) -> None:
    """Attach a stderr handler once; swap the file handler per-cycle when
    log_file is given (so each cycle in an N-cycle loop gets its own file)."""
    log.setLevel(logging.DEBUG)
    fmt = logging.Formatter("%(asctime)s %(levelname)s %(message)s", "%H:%M:%S")
    if not any(isinstance(h, logging.StreamHandler) and getattr(h, "stream", None) is sys.stderr for h in log.handlers):
        sh = logging.StreamHandler(sys.stderr)
        sh.setFormatter(fmt)
        sh.setLevel(logging.DEBUG)
        log.addHandler(sh)
    if log_file is not None:
        for h in list(log.handlers):
            if isinstance(h, logging.FileHandler):
                log.removeHandler(h)
                h.close()
        log_file.parent.mkdir(parents=True, exist_ok=True)
        fh = logging.FileHandler(log_file, encoding="utf-8")
        fh.setFormatter(fmt)
        fh.setLevel(logging.DEBUG)
        log.addHandler(fh)
        log.debug("file logging started at %s", log_file)


def run_logged(cmd: list[str], **kwargs) -> subprocess.CompletedProcess:
    """subprocess.run wrapper that logs command + exit. Use for non-redirected calls."""
    log.debug("run: %s", " ".join(cmd))
    proc = subprocess.run(cmd, **kwargs)
    log.debug("exit %d: %s", proc.returncode, Path(cmd[0]).name)
    return proc


def resolve_cohort_size(config_file: Path, env_value: str | None) -> tuple[int, str | None]:
    """Return (cohort_size, explicit_env_value).

    Precedence: env override > config.toml > built-in default.
    """
    explicit = env_value if env_value not in (None, "") else None
    if explicit is not None:
        return _validate_positive_int(explicit), explicit

    if config_file.is_file():
        for line in config_file.read_text(encoding="utf-8").splitlines():
            m = re.match(r"\s*cohort_size\s*=\s*(\d+)", line)
            if m:
                return _validate_positive_int(m.group(1)), None
    return DEFAULT_COHORT_SIZE, None


def _validate_positive_int(value: str) -> int:
    if not re.fullmatch(r"\d+", value) or int(value) < 1:
        die(f"COHORT_SIZE must be a positive integer, got: {value}")
    return int(value)


HYPOTHESIS_FILE_RE = re.compile(r"H\d{2}\.md")


def _first_canonical_line(body: str) -> str:
    """Return the first non-empty, non-header line of a hypothesis body."""
    for raw in body.splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        return line
    return ""


def resolve_hypotheses(
    env_value: str | None,
    cohort_size: int,
    explicit_cohort: str | None,
    *,
    scripts_dir: Path,
    cycle_dir: Path,
    cycle_num: str,
    python: str,
) -> tuple[list[str], int, bool]:
    """Return (hypotheses, effective_cohort_size, wrote_files).

    Precedence: explicit HYPOTHESES env wins (one Variant per non-empty line;
    caller scaffolds HNN.md from these strings); otherwise the wrapper invokes
    the agent which writes the files itself (caller re-reads them).
    """
    if env_value:
        hyps = [line for line in env_value.splitlines() if line]
        if explicit_cohort is not None and explicit_cohort != str(len(hyps)):
            print(
                f"note: HYPOTHESES has {len(hyps)} entries; COHORT_SIZE={explicit_cohort} ignored",
                file=sys.stderr,
            )
        return hyps, len(hyps), False

    writer = scripts_dir / "hypothesis-writer.py"
    log.info("invoking hypothesis-writer (N=%d, cycle=%s)", cohort_size, cycle_num)
    proc = subprocess.run(
        [python, str(writer), str(cohort_size)],
        capture_output=True,
        text=True,
        env={**os.environ, "CYCLE_NUM": cycle_num},
    )
    if proc.returncode != 0:
        sys.stderr.write(proc.stderr)
        die(f"hypothesis-writer failed (exit {proc.returncode})")

    hyp_dir = cycle_dir / "hypotheses"
    files = sorted(p for p in hyp_dir.iterdir() if HYPOTHESIS_FILE_RE.fullmatch(p.name))
    if len(files) != cohort_size:
        die(
            f"hypothesis-writer wrote {len(files)} H<NN>.md files in {hyp_dir}, "
            f"expected {cohort_size}"
        )
    hyps = [_first_canonical_line(p.read_text(encoding="utf-8")) for p in files]
    for p, line in zip(files, hyps):
        if not line:
            die(f"{p} has no canonical hypothesis line (first non-empty non-header)")
    return hyps, cohort_size, True


def next_cycle_num(cycles_dir: Path) -> str:
    cycles_dir.mkdir(parents=True, exist_ok=True)
    existing = [p.name for p in cycles_dir.iterdir() if re.fullmatch(r"\d{4}", p.name)]
    last = max((int(n) for n in existing), default=0)
    return f"{last + 1:04d}"


def scaffold_cohort(cycle_dir: Path, hypotheses: list[str], *, write_hyp_files: bool) -> None:
    """Write cohort.md (always) plus hypotheses/HNN.md (when not already
    written by the agent)."""
    hyp_dir = cycle_dir / "hypotheses"
    hyp_dir.mkdir(parents=True, exist_ok=True)

    lines = [f"# Cycle {cycle_dir.name} Cohort", "", f"Cohort size: {len(hypotheses)}", "", "## Hypotheses", ""]
    for i, text in enumerate(hypotheses, start=1):
        hid = f"H{i:02d}"
        lines.append(f"- [`{hid}`](hypotheses/{hid}.md)")
        if write_hyp_files:
            (hyp_dir / f"{hid}.md").write_text(f"# Hypothesis {hid}\n\n{text}\n", encoding="utf-8")
    (cycle_dir / "cohort.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


def _run_implementer(job: tuple[str, str, Path, Path]) -> tuple[str, int, Path]:
    vid, implementer, hyp_file, vdir = job
    log_path = vdir / "implementer.log"
    log.debug("%s: implementer %s %s -> %s", vid, implementer, hyp_file, log_path)
    with open(log_path, "wb") as f:
        proc = subprocess.run(
            [implementer, str(hyp_file), str(vdir / "impl")],
            stdout=f,
            stderr=subprocess.STDOUT,
        )
    log.debug("%s: implementer exit %d", vid, proc.returncode)
    return vid, proc.returncode, log_path


def fan_out_implementers(
    cycle_dir: Path,
    cohort_size: int,
    implementer: str,
) -> None:
    """Invoke VARIANT_IMPLEMENTER once per Variant in parallel. Aborts on any failure."""
    log.info("fan-out: %d implementer(s) in parallel (%s)", cohort_size, implementer)
    jobs = []
    for i in range(1, cohort_size + 1):
        vdir = variant_dir(cycle_dir, i)
        (vdir / "impl").mkdir(parents=True, exist_ok=True)
        (vdir / "eval" / "fitness").mkdir(parents=True, exist_ok=True)
        jobs.append((f"v{i:03d}", implementer, cycle_dir / "hypotheses" / f"H{i:02d}.md", vdir))

    failed = False
    with concurrent.futures.ThreadPoolExecutor(max_workers=cohort_size) as pool:
        for vid, code, log_path in pool.map(_run_implementer, jobs):
            if code != 0:
                print(f"error: variant-implementer for {vid} failed (exit {code}):", file=sys.stderr)
                sys.stderr.write(log_path.read_text(encoding="utf-8"))
                log.error("implementer %s failed (exit %d), see %s", vid, code, log_path)
                failed = True
    if failed:
        sys.exit(1)
    log.info("fan-out: all %d implementers succeeded", cohort_size)


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
        vdir = variant_dir(cycle_dir, i)
        log.info("[%s] gate-check + fitness (cycle %s)", vid, cycle_num)

        gate = run_logged(
            ["bash", str(scripts_dir / "gate-check.sh"), str(vdir / "impl"), cycle_num, vid]
        )
        if gate.returncode != 0:
            die(f"gate-check failed for {vid} in cycle {cycle_num}")

        fitness_dir = vdir / "eval" / "fitness"
        for plugin in plugins:
            out = fitness_dir / f"{plugin}.json"
            log.debug("[%s] plugin %s -> %s", vid, plugin, out)
            with open(out, "wb") as f:
                subprocess.run([python, str(scripts_dir / "fitness" / f"{plugin}.py")], stdout=f, check=True)
            if out.stat().st_size == 0:
                die(f"plugin {plugin} produced empty output for {vid} at {out}")
            try:
                json.loads(out.read_text(encoding="utf-8"))
            except json.JSONDecodeError:
                die(f"plugin {plugin} produced invalid JSON for {vid} at {out}")

        log.debug("[%s] aggregate fitness-vector", vid)
        with open(vdir / "eval" / "fitness-vector.json", "wb") as f:
            subprocess.run(
                [python, str(scripts_dir / "fitness-aggregate.py"), vid, str(fitness_dir)],
                stdout=f,
                check=True,
            )


def collect_fitness_vectors(cycle_dir: Path, cohort_size: int) -> list[dict]:
    vectors = []
    for i in range(1, cohort_size + 1):
        path = variant_dir(cycle_dir, i) / "eval" / "fitness-vector.json"
        vectors.append(json.loads(path.read_text(encoding="utf-8")))
    return vectors


def compute_pareto_front(scripts_dir: Path, python: str, fitness_vectors: list[dict]) -> list[str]:
    log.debug("pareto: %d fitness vector(s)", len(fitness_vectors))
    proc = subprocess.run(
        [python, str(scripts_dir / "pareto.py"), "-"],
        input=json.dumps(fitness_vectors),
        capture_output=True,
        text=True,
        check=True,
    )
    front = json.loads(proc.stdout)
    log.info("pareto front: %s", front)
    return front


def write_per_variant_insights(scripts_dir: Path, cycle_dir: Path, cohort_size: int, python: str) -> None:
    for i in range(1, cohort_size + 1):
        vid = f"v{i:03d}"
        vdir = variant_dir(cycle_dir, i)
        dim_summary = subprocess.run(
            [python, str(scripts_dir / "dim-summary.py"), str(vdir / "eval" / "fitness-vector.json")],
            capture_output=True,
            text=True,
            check=True,
        ).stdout.rstrip("\n")
        log.debug("[%s] insight: %s", vid, dim_summary)
        (vdir / "insight.md").write_text(
            f"# Variant {vid} Insight\n\nFitness: {dim_summary}\n",
            encoding="utf-8",
        )


def _parse_cycles_env(raw: str | None) -> int:
    if raw in (None, ""):
        return 1
    if not re.fullmatch(r"\d+", raw) or int(raw) < 1:
        die(f"EVOLVE_CYCLES must be a positive integer, got: {raw}")
    return int(raw)


def run_cycle() -> None:
    """Run a single evolve cycle end-to-end (allocate → scaffold → fan-out →
    gates → aggregate → seal → ledger). Reads env at call time so the outer
    N-cycle loop sees a fresh slot per iteration."""
    # --- Roots + env propagated to all subprocesses ---
    script_dir = Path(__file__).resolve().parent
    evolve_root = Path(os.environ.get("EVOLVE_ROOT") or script_dir.parent).resolve()
    repo_root = Path(os.environ.get("REPO_ROOT") or evolve_root.parent).resolve()
    os.environ["EVOLVE_ROOT"] = str(evolve_root)
    os.environ["REPO_ROOT"] = str(repo_root)

    worker_id = os.environ.get("WORKER_ID") or "local"
    os.environ["WORKER_ID"] = worker_id

    implementer = os.environ.get("VARIANT_IMPLEMENTER") or str(script_dir / "variant-implementer.sh")
    log.debug("evolve_root=%s repo_root=%s worker_id=%s", evolve_root, repo_root, worker_id)
    log.debug("implementer=%s", implementer)

    cohort_size, explicit_cohort = resolve_cohort_size(
        evolve_root / "config.toml", os.environ.get("COHORT_SIZE")
    )

    # --- Allocate next cycle slot ---
    cycles_dir = evolve_root / "cycles"
    cycle_num = next_cycle_num(cycles_dir)
    cycle_dir = cycles_dir / cycle_num
    ledger = evolve_root / "ledger" / "cycles.jsonl"

    if cycle_dir.is_dir():
        die(f"cycle {cycle_num} already exists at {cycle_dir}")

    # File handler under evolve/logs/ — outside cycle_dir so writes survive the seal step.
    setup_logging(evolve_root / "logs" / f"cycle-{cycle_num}.log")
    log.info("=== cycle %s allocated at %s ===", cycle_num, cycle_dir)

    # Prefer the project venv so fitness plugins / aggregator pick up pinned deps.
    python_bin = str(evolve_root / ".venv" / "bin" / "python3")
    if not (os.path.isfile(python_bin) and os.access(python_bin, os.X_OK)):
        python_bin = "python3"
    log.debug("python_bin=%s", python_bin)

    # The agent writes H<NN>.md into cycle_dir/hypotheses/; pre-create so the
    # adapter has a writable target.
    (cycle_dir / "hypotheses").mkdir(parents=True, exist_ok=True)

    hypotheses, cohort_size, wrote_files = resolve_hypotheses(
        os.environ.get("HYPOTHESES"),
        cohort_size,
        explicit_cohort,
        scripts_dir=script_dir,
        cycle_dir=cycle_dir,
        cycle_num=cycle_num,
        python=python_bin,
    )
    log.info("cohort_size=%d (explicit_env=%s, hypotheses=%d)", cohort_size, explicit_cohort, len(hypotheses))

    # --- Scaffold cohort, fan out implementers, gate + score each variant ---
    log.info("--- scaffold cohort ---")
    scaffold_cohort(cycle_dir, hypotheses, write_hyp_files=not wrote_files)
    log.info("--- fan out implementers ---")
    fan_out_implementers(cycle_dir, cohort_size, implementer)
    log.info("--- gates + fitness (sequential, cargo serializes target/) ---")
    run_gates_and_fitness(script_dir, cycle_dir, cycle_num, cohort_size, python_bin)

    # --- Aggregate cohort: fitness vectors → Pareto front → insights ---
    log.info("--- aggregate cohort ---")
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
        ),
        encoding="utf-8",
    )

    pareto_str = json.dumps(pareto_front)
    (cycle_dir / "cycle-insight.md").write_text(
        f"# Cycle {cycle_num} Insight\n\n"
        f"Cohort size: {cohort_size}\n"
        f"Pareto front: {pareto_str}\n\n"
        f"Evaluated {cohort_size} variant(s) with real fitness plugins "
        f"(hit-rate, per-op-time, metadata-size).\n",
        encoding="utf-8",
    )

    # --- Seal cycle dir (chmod -w, drop SEALED marker) ---
    log.info("--- seal cycle dir ---")
    run_logged(["bash", str(script_dir / "seal.sh"), str(cycle_dir)], check=True)

    # --- Append ledger row, then regenerate leaderboard from ledger + winners ---
    log.info("--- append ledger row ---")
    row = {
        "cycle": cycle_num,
        "worker_id": worker_id,
        "cycle_dir": f"evolve/cycles/{cycle_num}",
        "cohort_size": cohort_size,
        "pareto_front": pareto_front,
    }
    log.debug("ledger row: %s", row)
    subprocess.run(
        [python_bin, str(script_dir / "ledger-append.py"), str(ledger)],
        input=json.dumps(row),
        text=True,
        check=True,
    )

    log.info("--- regenerate leaderboard ---")
    run_logged(
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

    log.info("=== cycle %s complete (cohort=%d) ===", cycle_num, cohort_size)
    print(f"cycle {cycle_num} complete — sealed at {cycle_dir} (cohort={cohort_size})")


def main() -> None:
    # Stderr logging from the top; file handler attached per-cycle once cycle_num exists.
    setup_logging(None)
    n_cycles = _parse_cycles_env(os.environ.get("EVOLVE_CYCLES"))
    log.info("EVOLVE_CYCLES=%d", n_cycles)
    for i in range(1, n_cycles + 1):
        log.info("--- evolve loop: cycle %d/%d ---", i, n_cycles)
        run_cycle()


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as e:
        cmd = e.cmd[0] if isinstance(e.cmd, (list, tuple)) and e.cmd else str(e.cmd)
        die(f"{Path(cmd).name} failed (exit {e.returncode})")
