"""Cohort aggregation stage of the evolve loop."""

from __future__ import annotations

import json
import logging

from evolve._io import read_utf8, write_json
from evolve.aggregate.selectors import Selector
from evolve.identifiers import CycleId
from evolve.paths import CycleLayout, CycleVariantDir

log = logging.getLogger(__name__)


def compute_cycle_winners(
    variant_dirs: list[CycleVariantDir],
    selector: Selector,
) -> tuple[list[str], dict[str, float], list[dict]]:
    """Return (winners, scores, fitness_vectors) for the cohort."""

    fitness_vectors = _collect_fitness_vectors(variant_dirs)
    result = selector.select(fitness_vectors)
    log.info("cycle winners: %s", result.winners)
    return result.winners, result.scores, fitness_vectors


def write_cycle_winners_json(
    cycle_layout: CycleLayout,
    cycle_id: CycleId,
    winners: list[str],
    scores: dict[str, float],
    fitness_vectors: list[dict],
) -> None:
    """Write the cohort's `cycle-winners.json` file."""

    write_json(
        cycle_layout.cycle_winners_json,
        {
            "cycle": str(cycle_id),
            "winners": winners,
            "scores": scores,
            "fitness_vectors": fitness_vectors,
        },
    )


def _collect_fitness_vectors(variant_dirs: list[CycleVariantDir]) -> list[dict]:
    return [
        json.loads(read_utf8(v.fitness_vector_json))
        for v in variant_dirs
    ]
