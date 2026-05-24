#!/usr/bin/env python3
"""Measure the permanent baseline and write its fitness vector.

The output is a checked-in artefact; rerun and commit whenever the bench
config or fitness plugins change so cycle-1 hypotheses ground in numbers
comparable to live cohorts.
"""

import logging
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from src.config import load_configs  # noqa: E402
from src.fitness.runner import compute_fitness  # noqa: E402
from src.variant.gate_check import run_gates  # noqa: E402
from src.paths import Baseline, Layout  # noqa: E402


def main() -> None:
    logging.basicConfig(stream=sys.stdout, level=logging.INFO, format="%(message)s")

    script_dir = Path(__file__).resolve().parent
    evolve_root = script_dir.parent.resolve()
    repo_root = evolve_root.parent.resolve()

    cfg = load_configs(evolve_root)
    layout = Layout(evolve_root=evolve_root, outputs_root=evolve_root / "outputs")
    baseline = layout.sources.baseline

    if not baseline.segment_rs.is_file():
        raise FileNotFoundError(f"{baseline.segment_rs} not found - seed the baseline before measuring")
    if not baseline.manifest_toml.is_file():
        raise FileNotFoundError(f"{baseline.manifest_toml} not found - seed the baseline before measuring")

    baseline.fitness_dir.mkdir(parents=True, exist_ok=True)
    variant = Baseline()

    logging.info("measuring baseline at %s", baseline.root)
    if not run_gates(variant, layout=layout, repo_root=repo_root):
        raise RuntimeError("baseline failed gate-check")
    compute_fitness(variant, layout=layout, repo_root=repo_root, cfg=cfg)
    logging.info("baseline fitness measured - %s", baseline.fitness_vector_json)


if __name__ == "__main__":
    main()
