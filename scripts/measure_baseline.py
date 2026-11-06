#!/usr/bin/env python3
"""Measure the permanent baseline and write its fitness vector.

The output is a checked-in artefact; rerun and commit whenever the bench
config or fitness plugins change so cycle-1 hypotheses ground in numbers
comparable to live cohorts.

With `--evaluation-profile NAME` (or `fitness.evaluation_profile` in
config.toml) the baseline is scored on the manifest's
`[evaluation.profiles.NAME]` grid and written to
`baseline/eval/profiles/NAME/`, leaving the default anchor untouched.
"""

import argparse
import logging
import sys
from pathlib import Path

from evolve.config import load_configs
from evolve.fitness.runner import compute_fitness
from evolve.subject import load_subject
from evolve.variant.gates import run_gates
from evolve.paths import Baseline, Layout


def main() -> None:
    logging.basicConfig(stream=sys.stdout, level=logging.INFO, format="%(message)s")

    repo_root = Path(__file__).resolve().parent.parent

    parser = argparse.ArgumentParser(description="Measure a subject's baseline fitness vector.")
    # Optional positional subject override so all subjects can be re-measured in a
    # loop without editing config.toml between runs; falls back to config.toml.
    parser.add_argument("subject", nargs="?", default=None)
    parser.add_argument("--evaluation-profile", default=None, metavar="NAME",
                        help="Measure on the manifest's [evaluation.profiles.NAME] grid.")
    args = parser.parse_args()
    cfg = load_configs(repo_root, subject=args.subject, evaluation_profile=args.evaluation_profile)
    layout = Layout(
        repo_root=repo_root,
        outputs_root=cfg.framework.outputs_root,
        subject=cfg.framework.subject,
        evaluation_profile=cfg.fitness.evaluation_profile,
    )
    # Fail fast on an unknown profile, before building anything.
    load_subject(layout.sources.subject_dir).evaluation_for(cfg.fitness.evaluation_profile)
    baseline = layout.sources.baseline

    # Baseline is the subject as-is (empty overlay); the subject manifest
    # must exist for the harness to copytree.
    if not layout.sources.subject_manifest.is_file():
        raise FileNotFoundError(
            f"{layout.sources.subject_manifest} not found - check out the subject crate"
        )

    # Empty overlay -> the baseline crate builds unchanged.
    baseline.overlay_dir.mkdir(parents=True, exist_ok=True)
    variant = Baseline()

    logging.info(
        "measuring baseline at %s (evaluation profile: %s)",
        baseline.root, cfg.fitness.evaluation_profile or "default",
    )
    if not run_gates(variant, layout=layout, repo_root=repo_root):
        raise RuntimeError("baseline failed gate-check")
    compute_fitness(variant, layout=layout, repo_root=repo_root, cfg=cfg)
    logging.info("baseline fitness measured - %s", baseline.fitness_vector_json)


if __name__ == "__main__":
    main()
