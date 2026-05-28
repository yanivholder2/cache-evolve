#!/usr/bin/env python3
"""Measure the permanent baseline and write its fitness vector.

The output is a checked-in artefact; rerun and commit whenever the bench
config or fitness plugins change so cycle-1 hypotheses ground in numbers
comparable to live cohorts.
"""

import logging
import sys
from pathlib import Path

from evolve.config import load_configs
from evolve.fitness.runner import compute_fitness
from evolve.variant.gate_check import run_gates
from evolve.paths import Baseline, Layout


def main() -> None:
    logging.basicConfig(stream=sys.stdout, level=logging.INFO, format="%(message)s")

    repo_root = Path(__file__).resolve().parent.parent

    cfg = load_configs(repo_root)
    layout = Layout(repo_root=repo_root, outputs_root=repo_root / "outputs")
    baseline = layout.sources.baseline

    # Baseline is the subject as-is (empty overlay); the subject manifest
    # must exist for the harness to copytree.
    if not layout.sources.subject_manifest.is_file():
        raise FileNotFoundError(
            f"{layout.sources.subject_manifest} not found - check out the subject crate"
        )

    baseline.files_dir.mkdir(parents=True, exist_ok=True)
    baseline.fitness_dir.mkdir(parents=True, exist_ok=True)
    variant = Baseline()

    logging.info("measuring baseline at %s", baseline.root)
    if not run_gates(variant, layout=layout, repo_root=repo_root):
        raise RuntimeError("baseline failed gate-check")
    compute_fitness(variant, layout=layout, repo_root=repo_root, cfg=cfg)
    logging.info("baseline fitness measured - %s", baseline.fitness_vector_json)


if __name__ == "__main__":
    main()
