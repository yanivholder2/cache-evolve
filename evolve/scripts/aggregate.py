"""Cohort aggregation stage: collect fitness vectors, compute Pareto front,
write per-variant insights, winners.json, and cycle-insight.md."""

from __future__ import annotations

import json
import logging
from pathlib import Path

from dim_summary import format_dimensions
from pareto import compute_pareto_front

log = logging.getLogger(__name__)


def aggregate_cohort(
    cycle_dir: Path,
    variant_dirs: list[Path],
    cycle_num: str,
) -> list[str]:
    """Run the aggregation stage end-to-end. Returns the Pareto front
    (variant IDs) for the ledger row."""

    fitness_vectors = _collect_fitness_vectors(variant_dirs)
    pareto_front = compute_pareto_front(fitness_vectors)
    log.info("pareto front: %s", pareto_front)
    _write_per_variant_insights(variant_dirs, fitness_vectors)

    cohort_size = len(variant_dirs)
    (cycle_dir / "winners.json").write_text(
        json.dumps({
            "cycle": cycle_num,
            "pareto_front": pareto_front,
            "fitness_vectors": fitness_vectors,
        }),
        encoding="utf-8",
    )
    (cycle_dir / "cycle-insight.md").write_text(
        f"# Cycle {cycle_num} Insight\n\n"
        f"Cohort size: {cohort_size}\n"
        f"Pareto front: {json.dumps(pareto_front)}\n\n"
        f"Evaluated {cohort_size} variant(s) with real fitness plugins "
        f"(hit-rate, per-op-time, metadata-size).\n",
        encoding="utf-8",
    )
    return pareto_front


def _collect_fitness_vectors(variant_dirs: list[Path]) -> list[dict]:
    return [
        json.loads((vdir / "eval" / "fitness-vector.json").read_text(encoding="utf-8"))
        for vdir in variant_dirs
    ]


def _write_per_variant_insights(variant_dirs: list[Path], fitness_vectors: list[dict]) -> None:
    for vdir, fv in zip(variant_dirs, fitness_vectors):
        vid = vdir.name
        summary = format_dimensions(fv)
        log.debug("[%s] insight: %s", vid, summary)
        (vdir / "insight.md").write_text(
            f"# Variant {vid} Insight\n\nFitness: {summary}\n",
            encoding="utf-8",
        )
