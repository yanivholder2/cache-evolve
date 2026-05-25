#!/usr/bin/env python3
"""Evolve framework entry point. Runs one or more research cycles."""

from __future__ import annotations

import argparse
import concurrent.futures
import logging
import re
import sys
import threading
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
from src.aggregate.cohort import compute_cohort_pareto, write_winners_json  # noqa: E402
from src.config import Configs, ConfigError, load_configs  # noqa: E402
from src.cycle.leaderboard import regenerate as regenerate_leaderboard  # noqa: E402
from src.cycle.ledger_append import append_row  # noqa: E402
from src.cycle.seal import seal_cycle  # noqa: E402
from src.cycle.cycle_insight_writer import write_cycle_insight  # noqa: E402
from src.fitness.runner import compute_fitness  # noqa: E402
from src.hypothesis.writer import write_hypotheses  # noqa: E402
from src.identifiers import CYCLE_ID_WIDTH, CycleId, HypothesisId, VariantId  # noqa: E402
from src._io import read_utf8  # noqa: E402
from src.paths import CycleLayout, CycleVariant, CycleVariantDir, Layout  # noqa: E402
from src.variant.gate_check import run_gates  # noqa: E402
from src.variant import implementer as _implementer  # noqa: E402
from src.variant.variant_insight_writer import write_insight  # noqa: E402

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


class VariantStepFailed(Exception):
    """Sentinel: one step in a variant's pipeline failed; abort remaining steps."""


@contextmanager
def _variant_pipeline_step(name: str, variant_id: VariantId):
    """Log step success/failure for one variant; convert any exception into VariantStepFailed."""
    try:
        yield
    except Exception as exception:
        log.warning("[%s] %s failed: %s", variant_id, name, exception)
        raise VariantStepFailed from exception
    else:
        log.info("[%s] %s done", variant_id, name)


def _next_cycle_id(cycles_dir: Path) -> CycleId:
    """Return the next CycleId by scanning `cycles_dir` for existing NNNN subdirs.

    Not atomic — concurrent allocators may collide.
    """

    cycles_dir.mkdir(parents=True, exist_ok=True)
    cycle_re = re.compile(rf"\d{{{CYCLE_ID_WIDTH}}}")
    existing = [p.name for p in cycles_dir.iterdir() if cycle_re.fullmatch(p.name)]
    last = max((int(n) for n in existing), default=0)
    return CycleId(last + 1)


def _named_thread(name: str, fn, *args, **kwargs):
    """Rename the current thread to `name`, then call `fn`."""
    threading.current_thread().name = name
    return fn(*args, **kwargs)


def _run_variant_pipeline(
    *,
    variant_id: VariantId,
    cycle_id: CycleId,
    cycle_layout: CycleLayout,
    layout: Layout,
    repo_root: Path,
    adapter: LLMAdapter,
    cfg: Configs,
) -> bool:
    """Run implement, gate-check, fitness, insight for one variant. Soft-fail.

    Returns True iff all four steps succeed.
    """

    variant = CycleVariant(cycle_id, variant_id)
    variant_dir = cycle_layout.variant(variant_id)
    hypothesis_file = cycle_layout.hypothesis_file(HypothesisId(variant_id.value))

    try:
        with _variant_pipeline_step("implement", variant_id):
            _implementer.implement_variant(hypothesis_file, variant_dir, layout=layout, adapter=adapter)
        with _variant_pipeline_step("gate-check", variant_id):
            if not run_gates(variant, layout=layout, repo_root=repo_root):
                raise RuntimeError("rejected variant")
        with _variant_pipeline_step("fitness", variant_id):
            compute_fitness(variant, layout=layout, repo_root=repo_root, cfg=cfg)
        with _variant_pipeline_step("insight", variant_id):
            write_insight(
                cycle_layout.variant(variant_id),
                cycle_layout.hypothesis_file(HypothesisId(variant_id.value)),
                layout=layout, adapter=adapter,
            )
        return True
    except VariantStepFailed:
        return False


def _run_per_variant(
    *,
    cycle_layout: CycleLayout,
    cycle_id: CycleId,
    cohort_size: int,
    layout: Layout,
    repo_root: Path,
    adapter: LLMAdapter,
    cfg: Configs,
) -> list[CycleVariantDir]:
    """Fan out the full per-variant pipeline (implement, gate, fitness, insight) in parallel."""

    variant_ids = [VariantId(i) for i in range(1, cohort_size + 1)]

    pool_size = cohort_size if cfg.framework.max_workers == 0 else cfg.framework.max_workers
    if cfg.framework.max_workers > cohort_size:
        log.warning(
            "max_workers=%d exceeds cohort_size=%d; capping useful concurrency at cohort_size",
            cfg.framework.max_workers, cohort_size,
        )

    results: dict[VariantId, bool] = {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=pool_size) as pool:
        future_to_id = {
            pool.submit(
                _named_thread, str(vid), _run_variant_pipeline,
                variant_id=vid, cycle_id=cycle_id, cycle_layout=cycle_layout,
                layout=layout, repo_root=repo_root, adapter=adapter, cfg=cfg,
            ): vid
            for vid in variant_ids
        }
        for future in concurrent.futures.as_completed(future_to_id):
            vid = future_to_id[future]
            try:
                results[vid] = future.result()
            except Exception as exception:
                log.warning("[%s] dropped: %s", vid, exception)
                results[vid] = False

    return [cycle_layout.variant(vid) for vid in variant_ids if results.get(vid)]


def _run_cycle_level(
    *,
    cycle_layout: CycleLayout,
    cycle_id: CycleId,
    survivors: list[CycleVariantDir],
    cohort_size: int,
    layout: Layout,
    adapter: LLMAdapter,
    cfg: Configs,
) -> tuple[list[str], str]:
    """Subphase 1: pareto + winners.json AND cycle-insight-writer, in parallel.

    Returns (pareto_front, cycle_insight_body).
    """

    hypothesis_files = [
        cycle_layout.hypothesis_file(HypothesisId(i))
        for i in range(1, cohort_size + 1)
    ]

    pareto_front, fitness_vectors = compute_cohort_pareto(survivors)

    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
        winners_future = pool.submit(
            _named_thread, "winners",
            write_winners_json, cycle_layout, cycle_id, pareto_front, fitness_vectors,
        )
        cycle_insight_future = pool.submit(
            _named_thread, "cycle_insight",
            write_cycle_insight,
            cycle_layout,
            survivors,
            pareto_front,
            hypothesis_files=hypothesis_files,
            ledger=layout.outputs.ledger,
            adapter=adapter,
            digest_cycles=cfg.framework.digest_cycles,
        )
        winners_future.result()
        log.info("winners.json done")
        cycle_insight_future.result()
        log.info("cycle-insight-writer done")

    cycle_insight = read_utf8(cycle_layout.cycle_insight_md)
    return pareto_front, cycle_insight


def _run_post_cycle_insight(
    *,
    cycle_id: CycleId,
    cohort_size: int,
    pareto_front: list[str],
    cycle_insight: str,
    layout: Layout,
) -> None:
    """Subphase 2: ledger-append AND leaderboard-regen, in parallel."""

    row = {
        "cycle": str(cycle_id),
        "cycle_dir": layout.cycle_ledger_path(cycle_id),
        "cohort_size": cohort_size,
        "pareto_front": pareto_front,
        "cycle_insight": cycle_insight,
    }
    log.debug("ledger row keys: %s", sorted(row))

    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
        append_future = pool.submit(
            _named_thread, "ledger",
            append_row, layout.outputs.ledger, row,
        )
        leaderboard_future = pool.submit(
            _named_thread, "leaderboard",
            regenerate_leaderboard,
            layout=layout.outputs,
            output=layout.outputs.leaderboard_md,
        )
        append_future.result()
        log.info("ledger-append done")
        leaderboard_future.result()
        log.info("leaderboard-regen done")


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

        with _phase("per-variant"):
            survivors = _run_per_variant(
                cycle_layout=cycle_layout,
                cycle_id=cycle_id,
                cohort_size=cohort_size,
                layout=layout,
                repo_root=repo_root,
                adapter=adapter,
                cfg=cfg,
            )
            if not survivors:
                die(f"no variants survived in cycle {cycle_id}")

        with _phase("cycle-level"):
            pareto_front, cycle_insight = _run_cycle_level(
                cycle_layout=cycle_layout,
                cycle_id=cycle_id,
                survivors=survivors,
                cohort_size=cohort_size,
                layout=layout,
                adapter=adapter,
                cfg=cfg,
            )
            log.info("cycle %s pareto front: %s", cycle_id, pareto_front)
            _run_post_cycle_insight(
                cycle_id=cycle_id,
                cohort_size=cohort_size,
                pareto_front=pareto_front,
                cycle_insight=cycle_insight,
                layout=layout,
            )

        with _phase("seal"):
            seal_cycle(cycle_dir)

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
    if layout.outputs.cross_cycle_synthesis_md.exists():
        leftovers.append(str(layout.outputs.cross_cycle_synthesis_md))
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
    parser.add_argument("--max-workers", type=int, default=None,
                        help="Worker threads for per-variant fan-out (0 = match cohort_size).")
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
            max_workers=args.max_workers,
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
