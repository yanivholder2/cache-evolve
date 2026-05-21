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

import _log
from _log import LEVELS as LOG_LEVELS
from adapters import REGISTRY, LLMAdapter
from aggregate import aggregate_cohort
from gates_and_fitness import run_gates_and_fitness
from hypothesis_writer import write_hypotheses
from leaderboard import regenerate as regenerate_leaderboard
from ledger_append import append_row
from scaffold import write_cohort_md
from seal import seal_cycle
from variant_implementer import Implementer, fan_out

log = logging.getLogger("evolve")


def die(msg: str) -> NoReturn:
    log.error(msg)
    print(f"error: {msg}", file=sys.stderr)
    sys.exit(1)


def _variant_dir(cycle_dir: Path, i: int) -> Path:
    return cycle_dir / "variants" / f"v{i:03d}"


def _next_cycle_num(cycles_dir: Path) -> str:
    cycles_dir.mkdir(parents=True, exist_ok=True)
    existing = [p.name for p in cycles_dir.iterdir() if re.fullmatch(r"\d{4}", p.name)]
    last = max((int(n) for n in existing), default=0)
    return f"{last + 1:04d}"


def run_cycle(
    *,
    evolve_root: Path,
    repo_root: Path,
    cohort_size: int,
    adapter: LLMAdapter,
    outputs_root: Path,
    implementer: Implementer | None = None,
    log_level: str = "DEBUG",
) -> None:
    """Run a single evolve cycle end-to-end (allocate → scaffold → fan-out →
    gates → aggregate → seal → ledger).

    `evolve_root` is the read-only source root (config, traces, subject, harness).
    `outputs_root` is the read-write working dir for cycles, ledger, logs, and
    the rendered leaderboard.
    """

    cycles_dir = outputs_root / "cycles"
    cycle_num = _next_cycle_num(cycles_dir)
    cycle_dir = cycles_dir / cycle_num
    ledger = outputs_root / "ledger" / "cycles.jsonl"

    if cycle_dir.is_dir():
        die(f"cycle {cycle_num} already exists at {cycle_dir}")

    # File handler under outputs/logs/ — outside cycle_dir so writes survive the seal step.
    _log.setup(outputs_root / "logs" / f"cycle-{cycle_num}.log", level=log_level)
    log.info("=== cycle %s allocated at %s ===", cycle_num, cycle_dir)

    hypothesis_files = write_hypotheses(
        cohort_size,
        evolve_root=evolve_root,
        outputs_root=outputs_root,
        cycle_num=cycle_num,
        adapter=adapter,
    )
    log.info("cohort_size=%d", cohort_size)
    variant_dirs = [_variant_dir(cycle_dir, i) for i in range(1, cohort_size + 1)]

    log.info("--- scaffold cohort ---")
    write_cohort_md(cycle_dir, hypothesis_files)
    log.info("--- fan out implementers ---")
    fan_out(
        cycle_dir,
        cohort_size,
        evolve_root=evolve_root,
        outputs_root=outputs_root,
        adapter=adapter,
        implementer=implementer,
    )
    log.info("--- gates + fitness (sequential, cargo serializes target/) ---")
    try:
        run_gates_and_fitness(
            variant_dirs,
            cycle_num,
            evolve_root=evolve_root,
            outputs_root=outputs_root,
            repo_root=repo_root,
        )
    except RuntimeError as e:
        die(str(e))

    log.info("--- aggregate cohort ---")
    pareto_front = aggregate_cohort(cycle_dir, variant_dirs, cycle_num)

    log.info("--- seal cycle dir ---")
    seal_cycle(cycle_dir)

    log.info("--- append ledger row ---")
    row = {
        "cycle": cycle_num,
        "cycle_dir": f"evolve/outputs/cycles/{cycle_num}",
        "cohort_size": cohort_size,
        "pareto_front": pareto_front,
    }
    log.debug("ledger row: %s", row)
    append_row(ledger, row)

    log.info("--- regenerate leaderboard ---")
    regenerate_leaderboard(outputs_root=outputs_root, output=outputs_root / "leaderboard.md")

    log.info("=== cycle %s complete (cohort=%d) ===", cycle_num, cohort_size)
    print(f"cycle {cycle_num} complete — sealed at {cycle_dir} (cohort={cohort_size})")


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


def _resolve_adapter(*, flag: str | None, defaults: dict) -> LLMAdapter:
    name = _resolve(flag, defaults, key="ai_agent_tool", flag="--ai-agent-tool")
    if name not in REGISTRY:
        die(f"unknown ai_agent_tool {name!r}; choices: {sorted(REGISTRY)}")
    return REGISTRY[name]()


def _resolve_log_level(flag_value: str | None, defaults: dict) -> str:
    value = str(_resolve(flag_value, defaults, key="log_level", flag="--log-level")).upper()
    if value not in LOG_LEVELS:
        die(f"--log-level must be one of {LOG_LEVELS}, got: {value!r}")
    return value


def _resolve_outputs_root(flag_value: Path | None, defaults: dict, *, evolve_root: Path) -> Path:
    value = _resolve(flag_value, defaults, key="outputs_root", flag="--outputs-root")
    path = value if isinstance(value, Path) else Path(value)
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

    adapter = _resolve_adapter(flag=args.ai_agent_tool, defaults=defaults)
    outputs_root = _resolve_outputs_root(args.outputs_root, defaults, evolve_root=evolve_root)
    log_level = _resolve_log_level(args.log_level, defaults)

    _log.setup(None, level=log_level)
    log.debug("evolve_root=%s repo_root=%s", evolve_root, repo_root)
    log.debug("adapter=%s outputs_root=%s log_level=%s", type(adapter).__name__, outputs_root, log_level)
    log.info("cycles=%d cohort_size=%d", n_cycles, cohort_size)

    for i in range(1, n_cycles + 1):
        log.info("--- evolve loop: cycle %d/%d ---", i, n_cycles)
        run_cycle(
            evolve_root=evolve_root,
            repo_root=repo_root,
            cohort_size=cohort_size,
            adapter=adapter,
            outputs_root=outputs_root,
            log_level=log_level,
        )


if __name__ == "__main__":
    main()
