"""Gates + fitness stage: run G1+G2+G3 and three fitness plugins per variant.

Sequential is intentional: cargo serializes on the shared `<outputs_root>/target`
dir, so concurrent variants would block each other anyway. Parallel evolve
invocations with distinct `--outputs-root` get isolated target dirs.
"""

from __future__ import annotations

import json
import logging
from pathlib import Path

from fitness import hit_rate, metadata_size, per_op_time
from fitness_aggregate import aggregate
from gate_check import run_gates

log = logging.getLogger(__name__)


def run_gates_and_fitness(
    variant_dirs: list[Path],
    cycle_num: str,
    *,
    evolve_root: Path,
    outputs_root: Path,
    repo_root: Path,
) -> None:
    """Run gates + fitness plugins for every variant. Raises RuntimeError on
    the first gate failure (orchestrator decides how to surface it)."""

    rejection_log = outputs_root / "rejection-log.md"
    target_dir = outputs_root / "target"
    for vdir in variant_dirs:
        vid = vdir.name
        log.info("[%s] gate-check + fitness (cycle %s)", vid, cycle_num)

        if not run_gates(vdir / "impl", cycle_num, vid, repo_root=repo_root, rejection_log=rejection_log):
            raise RuntimeError(f"gate-check failed for {vid} in cycle {cycle_num}")

        fitness_dir = vdir / "eval" / "fitness"
        plugins = {
            "hit-rate": lambda: hit_rate.compute(repo_root=repo_root, target_dir=target_dir),
            "per-op-time": lambda: per_op_time.compute(evolve_root=evolve_root, target_dir=target_dir),
            "metadata-size": lambda: metadata_size.compute(repo_root=repo_root, target_dir=target_dir),
        }
        for name, fn in plugins.items():
            log.debug("[%s] plugin %s", vid, name)
            record = fn()
            (fitness_dir / f"{name}.json").write_text(json.dumps(record), encoding="utf-8")

        log.debug("[%s] aggregate fitness-vector", vid)
        fv = aggregate(vid, fitness_dir)
        (vdir / "eval" / "fitness-vector.json").write_text(json.dumps(fv), encoding="utf-8")
