#!/usr/bin/env python3
"""Measure the permanent baseline at evolve/subject/baseline/ and write its
fitness vector.

Runs G1+G2+G3 + fitness plugins against `evolve/subject/baseline/impl/` and
writes `evolve/subject/baseline/eval/fitness-vector.json`. The output is a
checked-in artefact; rerun this script (and commit the diff) whenever the
bench config or fitness plugins change, so cycle-1 hypotheses ground in
numbers comparable to live cohorts.
"""

import logging
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from src.gates_and_fitness import run_gates_and_fitness  # noqa: E402
from src.paths import Layout  # noqa: E402


def main() -> None:
    logging.basicConfig(stream=sys.stdout, level=logging.INFO, format="%(message)s")

    script_dir = Path(__file__).resolve().parent
    evolve_root = script_dir.parent.resolve()
    repo_root = evolve_root.parent.resolve()

    baseline_root = Layout(evolve_root=evolve_root).baseline.root
    layout = Layout(evolve_root=evolve_root, outputs_root=baseline_root / "outputs")
    baseline = layout.baseline

    if not baseline.segment_rs.is_file():
        raise FileNotFoundError(f"{baseline.segment_rs} not found - seed the baseline before measuring")
    if not baseline.manifest_toml.is_file():
        raise FileNotFoundError(f"{baseline.manifest_toml} not found - seed the baseline before measuring")

    baseline.fitness_dir.mkdir(parents=True, exist_ok=True)

    logging.info("measuring baseline at %s", baseline.root)
    run_gates_and_fitness([baseline], cycle_num="baseline", layout=layout, repo_root=repo_root)
    logging.info("baseline fitness measured - %s", baseline.fitness_vector_json)


if __name__ == "__main__":
    main()
