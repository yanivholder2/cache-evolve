#!/usr/bin/env python3
"""Evolve framework entry. Runs one or more research cycles. Each cycle fans
out a cohort of N Variants, runs real G1+G2+G3 gates and fitness plugins per
variant, aggregates a Pareto front, seals the cycle, and appends one ledger
row.

CLI:
  evolve.py [--cycles N] [--cohort-size N] [--ai-agent-tool <name>]
            [--outputs-root <path>]
"""

from __future__ import annotations

import argparse
import logging
import re
import sys
import tomllib
from pathlib import Path
from typing import NoReturn

# Script mode: put `evolve/` on sys.path so `from src import …` resolves.
# Import mode: caller sets up the path; skip to avoid leaking sibling dirs.
if __name__ == "__main__":
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from src import _log  # noqa: E402
from src._log import LEVELS as LOG_LEVELS  # noqa: E402
from src.adapters import REGISTRY, LLMAdapter  # noqa: E402
from src.aggregate import aggregate_cohort  # noqa: E402
from src.gates_and_fitness import run_gates_and_fitness  # noqa: E402
from src.hypotheses_writer import write_hypotheses  # noqa: E402
from src.leaderboard import regenerate as regenerate_leaderboard  # noqa: E402
from src.ledger_append import append_row  # noqa: E402
from src.paths import CYCLE_ID_WIDTH, Layout  # noqa: E402
from src.scaffold import write_cohort_md  # noqa: E402
from src.seal import seal_cycle  # noqa: E402
from src.variant_implementer import Implementer, fan_out  # noqa: E402

log = logging.getLogger("evolve")


def die(msg: str) -> NoReturn:
    log.error(msg)
    print(f"error: {msg}", file=sys.stderr)
    sys.exit(1)


def _next_cycle_num(cycles_dir: Path) -> str:
    """Return the next zero-padded cycle ID by scanning `cycles_dir` for
    existing `NNNN` subdirs. Not atomic — caller must guard against
    concurrent allocation collisions."""

    cycles_dir.mkdir(parents=True, exist_ok=True)
    cycle_re = re.compile(rf"\d{{{CYCLE_ID_WIDTH}}}")
    existing = [p.name for p in cycles_dir.iterdir() if cycle_re.fullmatch(p.name)]
    last = max((int(n) for n in existing), default=0)
    return f"{last + 1:0{CYCLE_ID_WIDTH}d}"


def run_cycle(
    *,
    layout: Layout,
    repo_root: Path,
    cohort_size: int,
    adapter: LLMAdapter,
    implementer: Implementer | None = None,
    log_level: str = "DEBUG",
) -> None:
    """Run a single evolve cycle end-to-end (allocate → scaffold → fan-out →
    gates → aggregate → seal → ledger).

    `layout` owns the evolve+outputs paths; `repo_root` is the git repo root
    (a sibling concern — used by gate-check and fitness binaries).
    """

    try:
        cycle_num = _next_cycle_num(layout.cycles_dir)
        cycle_layout = layout.cycle(cycle_num)
        cycle_dir = cycle_layout.root

        if cycle_dir.is_dir():
            die(f"cycle {cycle_num} already exists at {cycle_dir}")

        # File handler under outputs_root/logs/ — outside cycle_dir so writes survive the seal step.
        _log.setup(layout.cycle_log(cycle_num), level=log_level)
        log.info("=== cycle %s allocated at %s ===", cycle_num, cycle_dir)

        log.info("--- write hypotheses ---")
        hypothesis_files = write_hypotheses(
            cohort_size,
            layout=layout,
            cycle_num=cycle_num,
            adapter=adapter,
        )
        variant_layouts = [cycle_layout.variant(i) for i in range(1, cohort_size + 1)]

        log.info("--- scaffold cohort ---")
        write_cohort_md(cycle_layout, hypothesis_files)
        log.info("--- fan out implementers ---")
        fan_out(
            cycle_layout,
            cohort_size,
            layout=layout,
            adapter=adapter,
            implementer=implementer,
        )
        log.info("--- gates + fitness (sequential, cargo serializes target/) ---")
        try:
            run_gates_and_fitness(
                variant_layouts,
                cycle_num,
                layout=layout,
                repo_root=repo_root,
            )
        except RuntimeError as e:
            die(str(e))

        log.info("--- aggregate cohort ---")
        pareto_front = aggregate_cohort(cycle_layout, variant_layouts, cycle_num)

        log.info("--- seal cycle dir ---")
        seal_cycle(cycle_dir)

        log.info("--- append ledger row ---")
        row = {
            "cycle": cycle_num,
            "cycle_dir": layout.cycle_ledger_path(cycle_num, repo_root),
            "cohort_size": cohort_size,
            "pareto_front": pareto_front,
        }
        log.debug("ledger row: %s", row)
        append_row(layout.ledger, row)

        log.info("--- regenerate leaderboard ---")
        regenerate_leaderboard(layout=layout.outputs, output=layout.leaderboard_md)

        log.info("=== cycle %s complete (cohort=%d) ===", cycle_num, cohort_size)
        print(f"cycle {cycle_num} complete — sealed at {cycle_dir} (cohort={cohort_size})")
    except Exception:
        log.exception("run_cycle failed")
        raise


def _check_no_leftovers(layout: Layout) -> None:
    """Die if outputs_root has artifacts from previous runs. Ignores target/
    (cargo cache, intended to persist)."""

    leftovers = []
    if layout.cycles_dir.is_dir() and any(layout.cycles_dir.iterdir()):
        leftovers.append(str(layout.cycles_dir))
    if layout.ledger.exists():
        leftovers.append(str(layout.ledger))
    if layout.logs_dir.is_dir() and any(layout.logs_dir.iterdir()):
        leftovers.append(str(layout.logs_dir))
    if layout.leaderboard_md.exists():
        leftovers.append(str(layout.leaderboard_md))
    if layout.synthesis_md.exists():
        leftovers.append(str(layout.synthesis_md))
    if layout.rejection_log.exists():
        leftovers.append(str(layout.rejection_log))
    if leftovers:
        die(f"outputs_root has leftovers from previous runs: {', '.join(leftovers)}")


def _load_framework_config(evolve_root: Path) -> dict:
    cfg_path = evolve_root / "config.toml"
    if not cfg_path.is_file():
        log.debug("config.toml not found at %s; using built-in defaults", cfg_path)
        return {}
    with open(cfg_path, "rb") as f:
        return tomllib.load(f).get("framework", {})


def _build_arg_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="evolve.py", description="Run one or more evolve research cycles.")
    parser.add_argument("--cycles", type=int, default=None,
                        help="Number of cycles to run sequentially (overrides config.toml).")
    parser.add_argument("--cohort-size", type=int, default=None,
                        help="Variants per cycle (overrides config.toml).")
    parser.add_argument("--ai-agent-tool", choices=sorted(REGISTRY), default=None,
                        help="AI agent CLI to drive subagents.")
    parser.add_argument("--outputs-root", type=Path, default=None,
                        help="Read-write working dir for cycles/ledger/logs.")
    parser.add_argument("--log-level", choices=LOG_LEVELS, default=None,
                        type=str.upper,
                        help="Log level for stderr + cycle log file (overrides config.toml).")
    return parser


def _resolve(flag_value, defaults: dict, *, key: str, flag: str):
    """Pick the effective value: flag > config.toml > die."""
    if flag_value is not None:
        return flag_value
    if key in defaults:
        return defaults[key]
    die(f"{key} not set (pass {flag} or set framework.{key} in config.toml)")


def _resolve_positive_int(flag_value: int | None, defaults: dict, *, key: str, flag: str) -> int:
    value = int(_resolve(flag_value, defaults, key=key, flag=flag))
    if value < 1:
        die(f"{flag} must be a positive integer, got: {value}")
    return value


def _resolve_outputs_root(flag_value: Path | None, defaults: dict, *, evolve_root: Path) -> Path:
    path = Path(_resolve(flag_value, defaults, key="outputs_root", flag="--outputs-root"))
    if not path.is_absolute():
        path = evolve_root / path
    return path.resolve()


def main(argv: list[str] | None = None) -> None:
    args = _build_arg_parser().parse_args(argv)

    script_dir = Path(__file__).resolve().parent
    evolve_root = script_dir.parent.resolve()
    repo_root = evolve_root.parent.resolve()

    defaults = _load_framework_config(evolve_root)
    n_cycles = _resolve_positive_int(args.cycles, defaults, key="cycles", flag="--cycles")
    cohort_size = _resolve_positive_int(args.cohort_size, defaults, key="cohort_size", flag="--cohort-size")

    adapter_name = _resolve(args.ai_agent_tool, defaults, key="ai_agent_tool", flag="--ai-agent-tool")
    if adapter_name not in REGISTRY:
        die(f"--ai-agent-tool must be one of {sorted(REGISTRY)}, got: {adapter_name!r}")
    adapter = REGISTRY[adapter_name]()
    outputs_root = _resolve_outputs_root(args.outputs_root, defaults, evolve_root=evolve_root)
    log_level = str(_resolve(
        args.log_level, defaults,
        key="log_level", flag="--log-level",
    )).upper()
    if log_level not in LOG_LEVELS:
        die(f"--log-level must be one of {LOG_LEVELS}, got: {log_level!r}")
    layout = Layout(evolve_root=evolve_root, outputs_root=outputs_root)

    _log.setup(None, level=log_level)
    _check_no_leftovers(layout)
    log.debug("evolve_root=%s repo_root=%s", evolve_root, repo_root)
    log.debug("adapter=%s outputs_root=%s log_level=%s", type(adapter).__name__, outputs_root, log_level)
    log.info("cycles=%d cohort_size=%d", n_cycles, cohort_size)

    for i in range(1, n_cycles + 1):
        log.info("--- evolve loop: cycle %d/%d ---", i, n_cycles)
        run_cycle(
            layout=layout,
            repo_root=repo_root,
            cohort_size=cohort_size,
            adapter=adapter,
            log_level=log_level,
        )


if __name__ == "__main__":
    main()
