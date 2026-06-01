"""Global cross-cycle Pareto champions.

Unlike the cohort-local `winners.json` (best of one cycle's survivors), this
unions every evaluated variant across all cycles with the baseline, re-labels
each fitness vector with a globally-unique id, and recomputes the Pareto front
over that union. Recomputed after every cycle so dominated variants stay
visible alongside the standing champions."""

from __future__ import annotations

import json
import logging

from evolve._io import read_utf8, write_json
from evolve.aggregate.pareto import compute_pareto_front
from evolve.identifiers import CYCLE_ID_WIDTH, CycleId, VariantId
from evolve.paths import Baseline, CycleLayout, CycleVariant, Layout

log = logging.getLogger(__name__)


def compute_champions(layout: Layout) -> dict:
    """Union all evaluated variants with the baseline and compute the global front."""

    vectors = _collect_global_vectors(layout)
    if not vectors:
        return {"pareto_front": [], "fitness_vectors": [], "best_per_dimension": {}}

    pareto_front = compute_pareto_front(vectors)
    best_per_dimension = _best_per_dimension(vectors)
    return {
        "pareto_front": pareto_front,
        "fitness_vectors": vectors,
        "best_per_dimension": best_per_dimension,
    }


def write_champions(layout: Layout) -> dict:
    """Compute and persist the global champions artifact."""

    data = compute_champions(layout)
    write_json(layout.outputs.champions_json, data)
    log.info(
        "global champions: %d variants, front=%s",
        len(data["fitness_vectors"]),
        data["pareto_front"],
    )
    return data


def _collect_global_vectors(layout: Layout) -> list[dict]:
    """Read every fitness vector (baseline first, then all evaluated cycle
    variants), each re-labelled with a globally-unique variant id."""

    vectors: list[dict] = []

    baseline_json = layout.sources.baseline.fitness_vector_json
    if baseline_json.is_file():
        fitness_vector = _read_fitness_vector(baseline_json)
        vectors.append({**fitness_vector, "variant": str(Baseline())})

    cycles_dir = layout.outputs.cycles_dir
    if cycles_dir.is_dir():
        for cycle_subdir in sorted(cycles_dir.iterdir()):
            name = cycle_subdir.name
            if not (len(name) == CYCLE_ID_WIDTH and name.isdigit()):
                continue
            cycle_layout = layout.outputs.cycle(CycleId.from_str(name))
            vectors.extend(_collect_cycle_vectors(cycle_layout))

    return vectors


def _collect_cycle_vectors(cycle_layout: CycleLayout) -> list[dict]:
    cycle_id = cycle_layout.cycle_id
    variants_dir = cycle_layout.variants_dir
    if not variants_dir.is_dir():
        return []
    vectors: list[dict] = []
    for variant_subdir in sorted(variants_dir.iterdir()):
        try:
            variant_id = VariantId.from_str(variant_subdir.name)
        except ValueError:
            continue
        fitness_vector_json = cycle_layout.variant(variant_id).fitness_vector_json
        if not fitness_vector_json.is_file():
            continue
        fitness_vector = _read_fitness_vector(fitness_vector_json)
        vectors.append(
            {**fitness_vector, "variant": str(CycleVariant(cycle_id, variant_id))}
        )
    return vectors


def _best_per_dimension(vectors: list[dict]) -> dict:
    """For each dimension, the globally-unique variant id maximizing its score
    (higher-is-better dims use raw summary; lower-is-better use the negation)."""

    best: dict[str, dict] = {}
    for fitness_vector in vectors:
        for dimension in fitness_vector["dimensions"]:
            dim_name = dimension["dim"]
            summary = float(dimension["summary"])
            score = summary if dimension["higher_is_better"] else -summary
            current = best.get(dim_name)
            if current is None or score > current["_score"]:
                best[dim_name] = {
                    "variant": fitness_vector["variant"],
                    "summary": dimension["summary"],
                    "_score": score,
                }
    return {
        dim_name: {"variant": entry["variant"], "summary": entry["summary"]}
        for dim_name, entry in best.items()
    }


def _read_fitness_vector(path) -> dict:
    try:
        return json.loads(read_utf8(path))
    except json.JSONDecodeError as exception:
        raise ValueError(f"malformed fitness vector at {path}: {exception}") from exception
