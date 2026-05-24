#!/usr/bin/env python3
"""Evolve framework entry point. Runs one or more research cycles."""

from __future__ import annotations

import argparse
import logging
import re
import sys
from contextlib import contextmanager
from pathlib import Path
from typing import NoReturn

# Script mode adds `evolve/` to sys.path; import mode trusts the caller's path
# to avoid leaking sibling dirs.
if __name__ == "__main__":
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from src import _log  # noqa: E402
from src._log import LOG_LEVELS  # noqa: E402
from src.adapters import ADAPTER_REGISTRY, LLMAdapter  # noqa: E402
from src.aggregate.cohort import aggregate_cohort  # noqa: E402
from src.config import Configs, ConfigError, load_configs  # noqa: E402
from src.cycle.leaderboard import regenerate as regenerate_leaderboard  # noqa: E402
from src.cycle.ledger_append import append_row  # noqa: E402
from src.cycle.seal import seal_cycle  # noqa: E402
from src.fitness.runner import compute_fitness  # noqa: E402
from src.hypothesis.writer import write_hypotheses  # noqa: E402
from src.identifiers import CYCLE_ID_WIDTH, CycleId, VariantId  # noqa: E402
from src.paths import CycleVariant, Layout  # noqa: E402
from src.variant.gate_check import run_gates  # noqa: E402
from src.variant.implementer import implement_variants  # noqa: E402

log = logging.getLogger("evolve")


def die(msg: str) -> NoReturn:
    log.error(msg)
    print(f"error: {msg}", file=sys.stderr)
    sys.exit(1)


@contextmanager
def _phase(name: str):
    """Log a header for `name`, then a matching footer on exit.

    Footer reads "done" on clean exit, "failed" when an exception propagates.
    """
    log.info("--- %s ---", name)
    try:
        yield
    except BaseException:
        log.info("--- %s failed ---", name)
        raise
    else:
        log.info("--- %s done ---", name)


def _next_cycle_id(cycles_dir: Path) -> CycleId:
    """Return the next CycleId by scanning `cycles_dir` for existing NNNN subdirs.

    Not atomic — concurrent allocators may collide.
    """

    cycles_dir.mkdir(parents=True, exist_ok=True)
    cycle_re = re.compile(rf"\d{{{CYCLE_ID_WIDTH}}}")
    existing = [p.name for p in cycles_dir.iterdir() if cycle_re.fullmatch(p.name)]
    last = max((int(n) for n in existing), default=0)
    return CycleId(last + 1)


def run_cycle(
    *,
    layout: Layout,
    repo_root: Path,
    adapter: LLMAdapter,
    cfg: Configs,
) -> None:
    """Run a single evolve cycle end-to-end."""

    cohort_size = cfg.framework.cohort_size
    try:
        cycle_id = _next_cycle_id(layout.outputs.cycles_dir)
        cycle_layout = layout.outputs.cycle(cycle_id)
        cycle_dir = cycle_layout.root

        if cycle_dir.is_dir():
            die(f"cycle {cycle_id} already exists at {cycle_dir}")

        # Log file lives outside cycle_dir so writes survive the seal step.
        _log.setup(layout.outputs.cycle_log(cycle_id), level=cfg.framework.log_level)
        log.info("=== cycle %s allocated at %s ===", cycle_id, cycle_dir)
        log.info("cohort_size=%d adapter=%s", cohort_size, adapter.identity())

        with _phase("write hypotheses"):
            write_hypotheses(
                cohort_size,
                layout=layout,
                cycle_id=cycle_id,
                adapter=adapter,
            )

        with _phase("implement variants"):
            try:
                implement_variants(
                    cycle_layout,
                    cohort_size,
                    layout=layout,
                    adapter=adapter,
                )
            except RuntimeError as exception:
                log.error("implement variants failed", exc_info=exception)
                die(str(exception))

        with _phase("gates + fitness"):
            variants = [CycleVariant(cycle_id, VariantId(i)) for i in range(1, cohort_size + 1)]
            survivors = []
            for variant in variants:
                log.info("[%s] gate-check + fitness", variant)
                if not run_gates(
                    variant,
                    layout=layout, repo_root=repo_root,
                ):
                    log.warning("[%s] gate-check failed, skipping fitness", variant)
                    continue
                compute_fitness(variant, layout=layout, repo_root=repo_root, cfg=cfg)
                survivors.append(variant.resolve(layout))
            if not survivors:
                die(f"no variants survived gate-check in cycle {cycle_id}")

        with _phase("aggregate cohort"):
            pareto_front = aggregate_cohort(cycle_layout, survivors, cycle_id)

        with _phase("seal cycle dir"):
            seal_cycle(cycle_dir)

        with _phase("append ledger row"):
            row = {
                "cycle": str(cycle_id),
                "cycle_dir": layout.cycle_ledger_path(cycle_id),
                "cohort_size": cohort_size,
                "pareto_front": pareto_front,
            }
            log.debug("ledger row: %s", row)
            append_row(layout.outputs.ledger, row)

        with _phase("regenerate leaderboard"):
            regenerate_leaderboard(layout=layout.outputs, output=layout.outputs.leaderboard_md)

        log.info("=== cycle %s complete (cohort=%d) ===", cycle_id, cohort_size)
        print(f"cycle {cycle_id} complete — sealed at {cycle_dir} (cohort={cohort_size})")
    except Exception:
        log.exception("run_cycle failed")
        raise


def _check_no_leftovers(layout: Layout) -> None:
    """Die if outputs_root has artifacts from previous runs.

    Ignores target/ — cargo cache is intended to persist.
    """

    def has_non_gitkeep(d: Path) -> bool:
        return d.is_dir() and any(p.name != ".gitkeep" for p in d.iterdir())

    leftovers = []
    if has_non_gitkeep(layout.outputs.cycles_dir):
        leftovers.append(str(layout.outputs.cycles_dir))
    if layout.outputs.ledger.exists():
        leftovers.append(str(layout.outputs.ledger))
    if has_non_gitkeep(layout.outputs.logs_dir):
        leftovers.append(str(layout.outputs.logs_dir))
    if layout.outputs.leaderboard_md.exists():
        leftovers.append(str(layout.outputs.leaderboard_md))
    if layout.outputs.synthesis_md.exists():
        leftovers.append(str(layout.outputs.synthesis_md))
    if leftovers:
        die(f"outputs_root has leftovers from previous runs: {', '.join(leftovers)}")


def _build_arg_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="evolve.py",
        description="Run one or more evolve research cycles. Flags override config.toml.",
    )
    parser.add_argument("--cycles", type=int, default=None,
                        help="Number of cycles to run sequentially.")
    parser.add_argument("--cohort-size", type=int, default=None,
                        help="Variants per cycle.")
    parser.add_argument("--ai-agent-tool", choices=sorted(ADAPTER_REGISTRY), default=None,
                        help="AI agent CLI to drive subagents.")
    parser.add_argument("--outputs-root", type=Path, default=None,
                        help="Read-write working dir for cycles/ledger/logs.")
    parser.add_argument("--log-level", choices=LOG_LEVELS, default=None,
                        type=str.upper,
                        help="Log level for stderr + cycle log file.")
    parser.add_argument("--simulator-timeout-s", type=int, default=None,
                        help="Per-variant simulator invocation timeout (seconds).")
    return parser


def main(argv: list[str] | None = None) -> None:
    args = _build_arg_parser().parse_args(argv)

    script_dir = Path(__file__).resolve().parent
    evolve_root = script_dir.parent.resolve()
    repo_root = evolve_root.parent.resolve()

    try:
        cfg = load_configs(
            evolve_root,
            cycles=args.cycles,
            cohort_size=args.cohort_size,
            ai_agent_tool=args.ai_agent_tool,
            outputs_root=args.outputs_root,
            log_level=args.log_level,
            simulator_timeout_s=args.simulator_timeout_s,
        )
    except ConfigError as exception:
        die(str(exception))

    adapter = ADAPTER_REGISTRY[cfg.framework.ai_agent_tool]()
    layout = Layout(evolve_root=evolve_root, outputs_root=cfg.framework.outputs_root)

    _log.setup(None, level=cfg.framework.log_level)
    _check_no_leftovers(layout)
    log.debug("evolve_root=%s repo_root=%s", evolve_root, repo_root)
    log.debug(
        "adapter=%s outputs_root=%s log_level=%s",
        type(adapter).__name__, cfg.framework.outputs_root, cfg.framework.log_level,
    )
    log.info("cycles=%d cohort_size=%d", cfg.framework.cycles, cfg.framework.cohort_size)

    for i in range(1, cfg.framework.cycles + 1):
        log.info("--- evolve loop: cycle %d/%d ---", i, cfg.framework.cycles)
        run_cycle(layout=layout, repo_root=repo_root, adapter=adapter, cfg=cfg)


if __name__ == "__main__":
    main()
