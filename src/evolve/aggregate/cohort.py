"""Cohort aggregation stage of the evolve loop."""

from __future__ import annotations

import json
import logging

from evolve._io import read_utf8, write_json
from evolve.identifiers import CycleId
from evolve.paths import CycleLayout, CycleVariantDir
from evolve.aggregate.pareto import compute_pareto_front

log = logging.getLogger(__name__)


def compute_cycle_winners(
    variant_dirs: list[CycleVariantDir],
) -> tuple[list[str], list[dict]]:
    """Return (winners, fitness_vectors) for the cohort."""

    fitness_vectors = _collect_fitness_vectors(variant_dirs)
    winners = compute_pareto_front(fitness_vectors)
    log.info("pareto front: %s", winners)
    return winners, fitness_vectors


def write_cycle_winners_json(
    cycle_layout: CycleLayout,
    cycle_id: CycleId,
    winners: list[str],
    fitness_vectors: list[dict],
) -> None:
    """Write the cohort's `cycle-winners.json` file."""

    write_json(
        cycle_layout.cycle_winners_json,
        {
            "cycle": str(cycle_id),
            "winners": winners,
            "fitness_vectors": fitness_vectors,
        },
    )


def _collect_fitness_vectors(variant_dirs: list[CycleVariantDir]) -> list[dict]:
    return [
        json.loads(read_utf8(v.fitness_vector_json))
        for v in variant_dirs
    ]
