"""Cohort aggregation stage of the evolve loop."""

from __future__ import annotations

import json
import logging

from src._io import read_utf8, write_json
from src.identifiers import CycleId
from src.paths import CycleLayout, CycleVariantDir
from src.aggregate.pareto import compute_pareto_front

log = logging.getLogger(__name__)


def compute_cohort_pareto(
    variant_dirs: list[CycleVariantDir],
) -> tuple[list[str], list[dict]]:
    """Return (pareto_front, fitness_vectors) for the cohort."""

    fitness_vectors = _collect_fitness_vectors(variant_dirs)
    pareto_front = compute_pareto_front(fitness_vectors)
    log.info("pareto front: %s", pareto_front)
    return pareto_front, fitness_vectors


def write_winners_json(
    cycle_layout: CycleLayout,
    cycle_id: CycleId,
    pareto_front: list[str],
    fitness_vectors: list[dict],
) -> None:
    """Write the cohort's `winners.json` file."""

    write_json(
        cycle_layout.winners_json,
        {
            "cycle": str(cycle_id),
            "pareto_front": pareto_front,
            "fitness_vectors": fitness_vectors,
        },
    )


def _collect_fitness_vectors(variant_dirs: list[CycleVariantDir]) -> list[dict]:
    return [
        json.loads(read_utf8(v.fitness_vector_json))
        for v in variant_dirs
    ]
