"""Gates + fitness stage: run G1+G2+G3 and three fitness plugins per variant.

Sequential is intentional: cargo serializes on the shared `<outputs_root>/target`
dir, so concurrent variants would block each other anyway. Parallel evolve
invocations with distinct `--outputs-root` get isolated target dirs.
"""

from __future__ import annotations

import json
import logging
from pathlib import Path

from src.fitness import hit_rate, metadata_size, per_op_time
from src.fitness_aggregate import aggregate
from src.gate_check import run_gates
from src.identifiers import CycleId
from src.paths import Layout, VariantLayout

log = logging.getLogger(__name__)


def run_gates_and_fitness(
    variant_layouts: list[VariantLayout],
    cycle_id: CycleId,
    *,
    layout: Layout,
    repo_root: Path,
) -> None:
    """Run gates + fitness plugins for every variant. Raises RuntimeError on
    the first gate failure (orchestrator decides how to surface it)."""

    rejection_log = layout.rejection_log
    target_dir = layout.target_dir
    for variant_layout in variant_layouts:
        variant_id = variant_layout.variant_id
        log.info("[%s] gate-check + fitness (cycle %s)", variant_id, cycle_id)

        if not run_gates(variant_layout.impl_dir, cycle_id, variant_id, repo_root=repo_root, rejection_log=rejection_log):
            raise RuntimeError(f"gate-check failed for {variant_id} in cycle {cycle_id}")

        fitness_dir = variant_layout.fitness_dir
        fitness_dir.mkdir(parents=True, exist_ok=True)
        plugins = {
            "hit-rate": lambda: hit_rate.compute(repo_root=repo_root, target_dir=target_dir),
            "per-op-time": lambda: per_op_time.compute(evolve_root=layout.evolve_root, target_dir=target_dir),
            "metadata-size": lambda: metadata_size.compute(repo_root=repo_root, target_dir=target_dir),
        }
        for name, fn in plugins.items():
            log.debug("[%s] plugin %s", variant_id, name)
            record = fn()
            (fitness_dir / f"{name}.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")

        log.debug("[%s] aggregate fitness-vector", variant_id)
        fv = aggregate(str(variant_id), fitness_dir)
        variant_layout.fitness_vector_json.write_text(json.dumps(fv, indent=2) + "\n", encoding="utf-8")
