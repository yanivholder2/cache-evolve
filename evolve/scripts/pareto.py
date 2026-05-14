#!/usr/bin/env python3
"""Pareto front computer. Pure function — no I/O.

CLI: pareto.py <fitness_vectors.json>
Reads a JSON array of FitnessVectors from a file (or stdin if '-'),
prints the Pareto-front variant list as JSON to stdout.
"""

import json
import sys


def _score(dim: dict) -> float:
    """Normalize so higher is always better."""
    s = float(dim["summary"])
    return s if dim["higher_is_better"] else -s


def _dominates(a: dict, b: dict) -> bool:
    """True if a dominates b: at least as good in every dim, strictly better in one."""
    scores_a = {d["dim"]: _score(d) for d in a["dimensions"]}
    scores_b = {d["dim"]: _score(d) for d in b["dimensions"]}
    dims = set(scores_a) | set(scores_b)
    neg_inf = float("-inf")
    at_least_as_good = all(scores_a.get(d, neg_inf) >= scores_b.get(d, neg_inf) for d in dims)
    strictly_better = any(scores_a.get(d, neg_inf) > scores_b.get(d, neg_inf) for d in dims)
    return at_least_as_good and strictly_better


def compute_pareto_front(fitness_vectors: list) -> list:
    """Return variant names on the Pareto front (preserves input order)."""
    front = []
    for i, candidate in enumerate(fitness_vectors):
        dominated = any(
            _dominates(other, candidate)
            for j, other in enumerate(fitness_vectors)
            if j != i
        )
        if not dominated:
            front.append(candidate["variant"])
    return front


def main():
    path = sys.argv[1] if len(sys.argv) > 1 else "-"
    text = sys.stdin.read() if path == "-" else open(path).read()
    vectors = json.loads(text)
    print(json.dumps(compute_pareto_front(vectors)))


if __name__ == "__main__":
    main()
