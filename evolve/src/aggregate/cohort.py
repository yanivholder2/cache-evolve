"""Cohort aggregation stage of the evolve loop."""

from __future__ import annotations

import json
import logging

from src._io import read_utf8, write_utf8
from src.identifiers import CycleId
from src.paths import CycleLayout, CycleVariantDir
from src.aggregate.pareto import compute_pareto_front

log = logging.getLogger(__name__)


def aggregate_cohort(
    cycle_layout: CycleLayout,
    variant_dirs: list[CycleVariantDir],
    cycle_id: CycleId,
) -> list[str]:
    """Run the aggregation stage end-to-end. Returns the Pareto front as
    variant IDs."""

    fitness_vectors = _collect_fitness_vectors(variant_dirs)
    pareto_front = compute_pareto_front(fitness_vectors)
    log.info("pareto front: %s", pareto_front)
    _write_per_variant_insights(variant_dirs, fitness_vectors)

    cohort_size = len(variant_dirs)
    write_utf8(
        cycle_layout.winners_json,
        json.dumps({
            "cycle": str(cycle_id),
            "pareto_front": pareto_front,
            "fitness_vectors": fitness_vectors,
        }),
    )
    write_utf8(
        cycle_layout.cycle_insight_md,
        f"# Cycle {cycle_id} Insight\n\n"
        f"Cohort size: {cohort_size}\n"
        f"Pareto front: {json.dumps(pareto_front)}\n\n"
        f"Evaluated {cohort_size} variant(s) with real fitness plugins "
        f"(hit-rate, per-op-time, metadata-size).\n",
    )
    return pareto_front


def _collect_fitness_vectors(variant_dirs: list[CycleVariantDir]) -> list[dict]:
    return [
        json.loads(read_utf8(v.fitness_vector_json))
        for v in variant_dirs
    ]


def _write_per_variant_insights(variant_dirs: list[CycleVariantDir], fitness_vectors: list[dict]) -> None:
    for variant_dir, fv in zip(variant_dirs, fitness_vectors):
        summary = ", ".join(f"{d['dim']}={d['summary']:.4g}" for d in fv.get("dimensions", []))
        variant_id = variant_dir.variant_id
        log.debug("[%s] insight: %s", variant_id, summary)
        write_utf8(
            variant_dir.insight_md,
            f"# Variant {variant_id} Insight\n\nFitness: {summary}\n",
        )
