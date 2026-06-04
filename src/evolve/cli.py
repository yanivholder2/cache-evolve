#!/usr/bin/env python3
"""Evolve framework entry point. Runs one or more research cycles."""

from __future__ import annotations

import argparse
import concurrent.futures
import json
import logging
import re
import sys
import threading
import time
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path
from typing import NoReturn

from evolve import _log
from evolve._log import LOG_LEVELS
from evolve.llm import (
    ADAPTER_REGISTRY,
    LLMAdapter,
    format_token_totals,
    snapshot_token_totals,
)
from evolve.aggregate.cross_cycle_winners import write_cross_cycle_winners
from evolve.aggregate.cohort import compute_cycle_winners, write_cycle_winners_json
from evolve.aggregate.selectors import Selector, build_selector
from evolve.config import AuthoringScope, Configs, ConfigError, load_configs
from evolve.cycle.leaderboard import regenerate as regenerate_leaderboard
from evolve.cycle.ledger_append import append_row
from evolve.cycle.seal import seal_cycle
from evolve.cycle.cross_cycle_insight_writer import write_cross_cycle_insight
from evolve.cycle.cycle_insight_writer import write_cycle_insight
from evolve.fitness.runner import compute_fitness
from evolve.hypothesis.writer import write_hypotheses
from evolve.identifiers import CYCLE_ID_WIDTH, CycleId, HypothesisId, VariantId
from evolve._io import read_utf8
from evolve.paths import CycleLayout, CycleVariant, CycleVariantDir, Layout
from evolve.subject import Subject, load_subject
from evolve.variant.implementer import _latest_rejection_entry
from evolve.variant.repair_loop import VariantStepFailed, implement_with_repair
from evolve.variant.variant_insight_writer import write_insight

# Explicit name (not __name__): this module runs as "__main__" under
# `python -m evolve.cli`, which would escape the "evolve" logging namespace.
log = logging.getLogger("evolve.cli")


def die(msg: str) -> NoReturn:
    log.error(msg)
    print(f"error: {msg}", file=sys.stderr)
    sys.exit(1)


def _require_scope_supported(subject: Subject, authoring_scope: AuthoringScope) -> None:
    """A subject with no extension points cannot be evolved under extension-points scope."""
    if authoring_scope is AuthoringScope.EXTENSION_POINTS and not subject.extension_points:
        die(
            f"subject {subject.name!r} declares no extension points, so authoring scope "
            f"'extension-points' would reject every variant; rerun with --authoring-scope full-autonomy"
        )


def _validate_active_dimensions(cfg: Configs, subject: Subject, layout: Layout) -> None:
    """Cross-cutting boundary guard, run once before cycle 0.

    Lives here, not in config.py, because it crosses the framework/subject seam:
    config.py stays subject-agnostic. Every active fitness dimension must be one
    the subject can measure (capability declared in its manifest) and must be
    present in the baseline anchor, or the cross-cycle union and leaderboard
    would compare incomparable numbers.
    """

    requires_table = {"per-op-time": "[bench]", "metadata-size": "[metadata]"}
    capable = {
        "hit-rate": True,
        "per-op-time": subject.bench is not None,
        "metadata-size": subject.metadata is not None,
    }
    for dim in cfg.fitness.dimensions:
        if dim not in capable:
            die(
                f"fitness.dimensions: unknown dimension {dim!r}; "
                f"known dimensions: {sorted(capable)}"
            )
        if not capable[dim]:
            die(
                f"fitness.dimensions: dimension {dim!r} requires the subject manifest "
                f"{requires_table[dim]} table, which subject {subject.name!r} does not declare"
            )

    baseline_dims = _baseline_measured_dimensions(layout)
    missing = [dim for dim in cfg.fitness.dimensions if dim not in baseline_dims]
    if missing:
        die(
            f"fitness.dimensions {missing} are not measured in the baseline anchor "
            f"(measured: {sorted(baseline_dims)}); re-measure the baseline with "
            f"scripts/measure_baseline.py"
        )


def _baseline_measured_dimensions(layout: Layout) -> set[str]:
    """The dimension names present in the subject's baseline fitness vector."""
    path = layout.sources.baseline.fitness_vector_json
    if not path.is_file():
        die(f"baseline fitness vector not found at {path}")
    data = json.loads(read_utf8(path))
    return {dimension["dim"] for dimension in data.get("dimensions", [])}


@contextmanager
def _phase(name: str, timings: dict[str, float] | None = None):
    """Log a header for `name`, then a matching footer with elapsed time on exit.

    Footer reads "done" on clean exit, "failed" when an exception propagates.
    When `timings` is provided, the elapsed seconds are recorded under `name`
    so the cycle can report a per-phase breakdown.
    """
    log.info("--- %s ---", name)
    started = time.monotonic()
    try:
        yield
    except BaseException:
        elapsed = time.monotonic() - started
        log.info("--- %s failed (%.1fs) ---", name, elapsed)
        if timings is not None:
            timings[name] = elapsed
        raise
    else:
        elapsed = time.monotonic() - started
        log.info("--- %s done (%.1fs) ---", name, elapsed)
        if timings is not None:
            timings[name] = elapsed


@contextmanager
def _variant_pipeline_step(name: str, variant_id: VariantId):
    """Log step success/failure for one variant; convert any exception into VariantStepFailed.

    Already-tagged VariantStepFailed (e.g. from gate-check naming a specific
    gate) propagates as-is so the survival summary keeps the finer reason.
    """
    try:
        yield
    except VariantStepFailed as exception:
        log.warning("[%s] %s failed: %s", variant_id, name, exception.reason)
        raise
    except Exception as exception:
        log.warning("[%s] %s failed: %s", variant_id, name, exception)
        raise VariantStepFailed(name, detail=str(exception)) from exception
    else:
        log.info("[%s] %s done", variant_id, name)


@dataclass(frozen=True)
class VariantOutcome:
    """Per-variant result threaded out of the per-variant fan-out for the
    cycle-level survival summary and the cycle-insight writer."""

    variant_id: VariantId
    hypothesis_id: HypothesisId
    survived: bool
    reason: str | None  # None iff survived; e.g. "gate:contract gate", "implement"
    detail: str | None  # underlying cause (gate body, scope path, "no overlay", exception text); None iff survived


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
) -> VariantOutcome:
    """Run implement-and-gate (with self-repair), fitness, insight for one
    variant. Soft-fail.

    Returns a `VariantOutcome` carrying survival + first-failing-step reason.
    """

    variant = CycleVariant(cycle_id, variant_id)
    hypothesis_id = HypothesisId(variant_id.value)
    hypothesis_file = cycle_layout.hypothesis_file(hypothesis_id)

    attempts = cfg.framework.implementation_attempts
    try:
        with _variant_pipeline_step("implement-and-gate", variant_id):
            gates_result = implement_with_repair(
                variant, hypothesis_file, layout=layout, repo_root=repo_root,
                adapter=adapter, authoring_scope=cfg.framework.authoring_scope,
                hypothesis_id=hypothesis_id, attempts=attempts,
            )
            if not gates_result:
                rejection_detail = _latest_rejection_entry(
                    variant.resolve(layout).rejection_md
                )
                raise VariantStepFailed(
                    f"gate:{gates_result.first_failed_gate} ({attempts}/{attempts} attempts)",
                    detail=rejection_detail,
                )
        with _variant_pipeline_step("fitness", variant_id):
            compute_fitness(variant, layout=layout, repo_root=repo_root, cfg=cfg)
        with _variant_pipeline_step("insight", variant_id):
            write_insight(
                cycle_layout.variant(variant_id),
                hypothesis_file,
                layout=layout, adapter=adapter,
            )
        return VariantOutcome(variant_id, hypothesis_id, True, None, detail=None)
    except VariantStepFailed as exception:
        return VariantOutcome(
            variant_id, hypothesis_id, False,
            reason=exception.reason, detail=exception.detail,
        )


def _run_per_variant(
    *,
    cycle_layout: CycleLayout,
    cycle_id: CycleId,
    cohort_size: int,
    layout: Layout,
    repo_root: Path,
    adapter: LLMAdapter,
    cfg: Configs,
) -> tuple[list[CycleVariantDir], list[VariantOutcome]]:
    """Fan out the full per-variant pipeline (implement, gate, fitness, insight) in parallel.

    Returns `(survivors, outcomes)` so the caller can emit a per-cycle
    survival summary that names which step rejected each variant.
    """

    variant_ids = [VariantId(i) for i in range(1, cohort_size + 1)]

    pool_size = cohort_size if cfg.framework.max_workers == 0 else cfg.framework.max_workers
    if cfg.framework.max_workers > cohort_size:
        log.warning(
            "max_workers=%d exceeds cohort_size=%d; capping useful concurrency at cohort_size",
            cfg.framework.max_workers, cohort_size,
        )

    outcomes_by_id: dict[VariantId, VariantOutcome] = {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=pool_size) as pool:
        future_to_id = {
            pool.submit(
                _named_thread, str(variant_id), _run_variant_pipeline,
                variant_id=variant_id, cycle_id=cycle_id, cycle_layout=cycle_layout,
                layout=layout, repo_root=repo_root, adapter=adapter, cfg=cfg,
            ): variant_id
            for variant_id in variant_ids
        }
        for future in concurrent.futures.as_completed(future_to_id):
            variant_id = future_to_id[future]
            try:
                outcomes_by_id[variant_id] = future.result()
            except Exception as exception:
                log.warning("[%s] dropped: %s", variant_id, exception)
                outcomes_by_id[variant_id] = VariantOutcome(
                    variant_id, HypothesisId(variant_id.value), False,
                    reason=f"dropped:{type(exception).__name__}", detail=str(exception),
                )

    outcomes = [outcomes_by_id[variant_id] for variant_id in variant_ids]
    survivors = [cycle_layout.variant(o.variant_id) for o in outcomes if o.survived]
    return survivors, outcomes


def _log_survival_summary(cycle_id: CycleId, cohort_size: int, outcomes: list[VariantOutcome]) -> None:
    """Single INFO line summarising who survived and who failed where.

    Recovering survival used to require ls-ing rejection.md across variants;
    this line surfaces an outlier cycle (e.g. survivors=2/5) at a glance.
    """

    survived = sum(1 for o in outcomes if o.survived)
    rejected = {str(o.variant_id): (o.reason or "?") for o in outcomes if not o.survived}
    log.info(
        "cycle %s survivors=%d/%d rejected=%s",
        cycle_id, survived, cohort_size, rejected,
    )


def _run_cycle_level(
    *,
    cycle_layout: CycleLayout,
    cycle_id: CycleId,
    survivors: list[CycleVariantDir],
    outcomes: list[VariantOutcome],
    cohort_size: int,
    layout: Layout,
    adapter: LLMAdapter,
    selector: Selector,
    cfg: Configs,
) -> tuple[list[str], str]:
    """Subphase 1: winner selection + cycle-winners.json AND cycle-insight-writer, in parallel.

    Returns (winners, cycle_insight_body).
    """

    hypothesis_files = [
        cycle_layout.hypothesis_file(HypothesisId(i))
        for i in range(1, cohort_size + 1)
    ]

    rejected = [
        {"variant_id": str(o.variant_id), "reason": o.reason, "detail": o.detail}
        for o in outcomes
        if not o.survived
    ]

    winners, scores, fitness_vectors = compute_cycle_winners(survivors, selector)

    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
        winners_future = pool.submit(
            _named_thread, "winners",
            write_cycle_winners_json, cycle_layout, cycle_id, winners, scores, fitness_vectors,
        )
        cycle_insight_future = pool.submit(
            _named_thread, "cycle_insight",
            write_cycle_insight,
            cycle_layout,
            survivors,
            winners,
            hypothesis_files=hypothesis_files,
            ledger=layout.outputs.ledger,
            adapter=adapter,
            digest_cycles=cfg.framework.digest_cycles,
            rejected=rejected,
        )
        winners_future.result()
        log.info("cycle-winners.json done")
        cycle_insight_future.result()
        log.info("cycle-insight-writer done")

    cycle_insight = read_utf8(cycle_layout.cycle_insight_md)
    return winners, cycle_insight


def _run_post_cycle_insight(
    *,
    cycle_id: CycleId,
    cohort_size: int,
    winners: list[str],
    cycle_insight: str,
    layout: Layout,
    selector: Selector,
) -> None:
    """Subphase 2: ledger-append AND leaderboard-regen, in parallel."""

    write_cross_cycle_winners(layout, selector)
    log.info("cross-cycle-winners.json done")

    row = {
        "cycle": str(cycle_id),
        "cycle_dir": layout.cycle_ledger_path(cycle_id),
        "cohort_size": cohort_size,
        "winners": winners,
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

    _require_scope_supported(
        load_subject(layout.sources.subject_dir), cfg.framework.authoring_scope
    )
    selector = build_selector(
        method=cfg.selection.method,
        winners_count=cfg.selection.winners_count,
        dimensions=cfg.fitness.dimensions,
        weights=cfg.selection.weights,
    )

    cohort_size = cfg.framework.cohort_size
    timings: dict[str, float] = {}
    cycle_started = time.monotonic()
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

        with _phase("write hypotheses", timings):
            write_hypotheses(
                cohort_size,
                layout=layout,
                cycle_id=cycle_id,
                adapter=adapter,
                authoring_scope=cfg.framework.authoring_scope,
                attempts=cfg.framework.hypothesis_attempts,
            )

        with _phase("per-variant", timings):
            survivors, outcomes = _run_per_variant(
                cycle_layout=cycle_layout,
                cycle_id=cycle_id,
                cohort_size=cohort_size,
                layout=layout,
                repo_root=repo_root,
                adapter=adapter,
                cfg=cfg,
            )
            _log_survival_summary(cycle_id, cohort_size, outcomes)
            if not survivors:
                die(f"no variants survived in cycle {cycle_id}")

        with _phase("cycle-level", timings):
            winners, cycle_insight = _run_cycle_level(
                cycle_layout=cycle_layout,
                cycle_id=cycle_id,
                survivors=survivors,
                outcomes=outcomes,
                cohort_size=cohort_size,
                layout=layout,
                adapter=adapter,
                selector=selector,
                cfg=cfg,
            )
            log.info("cycle %s winners: %s", cycle_id, winners)
            _run_post_cycle_insight(
                cycle_id=cycle_id,
                cohort_size=cohort_size,
                winners=winners,
                cycle_insight=cycle_insight,
                layout=layout,
                selector=selector,
            )

        with _phase("cross-cycle-insights", timings):
            try:
                write_cross_cycle_insight(cycle_layout, layout=layout, adapter=adapter)
            except Exception:
                log.warning("cross-cycle-insight writer failed; cycle proceeds", exc_info=True)

        # with _phase("seal", timings):
        #     seal_cycle(cycle_dir)

        cycle_elapsed = time.monotonic() - cycle_started
        log.info(
            "cycle %s timings: total=%.1fs %s",
            cycle_id, cycle_elapsed,
            " ".join(f"{name}={timings[name]:.1f}s" for name in timings),
        )
        log.info("cycle %s cumulative tokens: %s", cycle_id, format_token_totals(snapshot_token_totals()))
        log.info("=== cycle %s complete (cohort=%d) ===", cycle_id, cohort_size)
        # print(f"cycle {cycle_id} complete — sealed at {cycle_dir} (cohort={cohort_size})")
    except Exception:
        log.exception("run_cycle failed")
        raise


def _build_arg_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="evolve",
        description="Run one or more evolve research cycles. Flags override config.toml.",
    )
    parser.add_argument("--subject", type=str, default=None,
                        help="Subject under evolution (dir under inputs/subjects/).")
    parser.add_argument("--cycles", type=int, default=None,
                        help="Number of cycles to run sequentially.")
    parser.add_argument("--cohort-size", type=int, default=None,
                        help="Variants per cycle.")
    parser.add_argument("--ai-agent-tool", choices=sorted(ADAPTER_REGISTRY), default=None,
                        help="AI agent CLI to drive subagents.")
    parser.add_argument("--log-level", choices=LOG_LEVELS, default=None,
                        type=str.upper,
                        help="Log level for stderr + cycle log file.")
    parser.add_argument("--simulator-timeout-s", type=int, default=None,
                        help="Per-variant simulator invocation timeout (seconds).")
    parser.add_argument("--max-workers", type=int, default=None,
                        help="Worker threads for per-variant fan-out (0 = match cohort_size).")
    parser.add_argument("--authoring-scope", type=AuthoringScope, choices=list(AuthoringScope), default=None,
                        help="Write surface the variant-implementer agent may author.")
    return parser


def main(argv: list[str] | None = None) -> None:
    args = _build_arg_parser().parse_args(argv)

    repo_root = Path(__file__).resolve().parents[2]

    try:
        cfg = load_configs(
            repo_root,
            cycles=args.cycles,
            cohort_size=args.cohort_size,
            ai_agent_tool=args.ai_agent_tool,
            log_level=args.log_level,
            simulator_timeout_s=args.simulator_timeout_s,
            max_workers=args.max_workers,
            authoring_scope=args.authoring_scope,
            subject=args.subject,
        )
    except ConfigError as exception:
        die(str(exception))

    adapter = ADAPTER_REGISTRY[cfg.framework.ai_agent_tool]()
    layout = Layout(repo_root=repo_root, outputs_root=cfg.framework.outputs_root, subject=cfg.framework.subject)
    _validate_active_dimensions(cfg, load_subject(layout.sources.subject_dir), layout)

    _log.setup(None, level=cfg.framework.log_level)
    log.debug("repo_root=%s", repo_root)
    log.debug(
        "adapter=%s outputs_root=%s log_level=%s",
        type(adapter).__name__, cfg.framework.outputs_root, cfg.framework.log_level,
    )
    log.info("cycles=%d cohort_size=%d", cfg.framework.cycles, cfg.framework.cohort_size)

    for i in range(1, cfg.framework.cycles + 1):
        log.info("--- evolve loop: cycle %d/%d ---", i, cfg.framework.cycles)
        run_cycle(layout=layout, repo_root=repo_root, adapter=adapter, cfg=cfg)

    log.info("run-level cumulative tokens: %s", format_token_totals(snapshot_token_totals()))


if __name__ == "__main__":
    main()
