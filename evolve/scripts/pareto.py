#!/usr/bin/env python3
"""Pareto front computer.

CLI: pareto.py <fitness_vectors.json>
Reads a JSON array of FitnessVectors from a file (or stdin if '-'),
prints the Pareto-front variant list as JSON to stdout.
"""

import json
import math
import sys


def _score(dim: dict) -> float:
    """Normalize so higher is always better. Raises if `summary` is not finite."""
    s = float(dim["summary"])
    if not math.isfinite(s):
        raise ValueError(f"non-finite summary for dim {dim.get('dim')!r}: {s}")
    return s if dim["higher_is_better"] else -s


def _build_scores(fitness_vectors: list) -> list:
    """Return [(variant, {dim: score}), ...]; raises if dim sets differ across vectors."""
    if not fitness_vectors:
        return []
    scored = [(fv["variant"], {d["dim"]: _score(d) for d in fv["dimensions"]}) for fv in fitness_vectors]
    expected = set(scored[0][1].keys())
    for variant, scores in scored[1:]:
        if set(scores.keys()) != expected:
            raise ValueError(
                f"variant {variant!r} has dim set {sorted(scores)} but expected {sorted(expected)}; "
                f"Pareto requires uniform dimensions across all vectors"
            )
    return scored


def _dominates(scores_a: dict, scores_b: dict) -> bool:
    """True if `a` dominates `b`: at least as good in every dim, strictly better in one."""
    at_least_as_good = all(scores_a[d] >= scores_b[d] for d in scores_a)
    strictly_better = any(scores_a[d] > scores_b[d] for d in scores_a)
    return at_least_as_good and strictly_better


def compute_pareto_front(fitness_vectors: list) -> list:
    """Return variant names on the Pareto front (preserves input order)."""
    scored = _build_scores(fitness_vectors)
    front = []
    for i, (variant, scores_a) in enumerate(scored):
        dominated = any(_dominates(scores_b, scores_a) for j, (_, scores_b) in enumerate(scored) if j != i)
        if not dominated:
            front.append(variant)
    return front


def main():
    path = sys.argv[1] if len(sys.argv) > 1 else "-"
    if path == "-":
        text = sys.stdin.read()
    else:
        with open(path) as f:
            text = f.read()
    try:
        vectors = json.loads(text)
    except json.JSONDecodeError as e:
        print(f"error: invalid JSON in {path}: {e}", file=sys.stderr)
        sys.exit(1)
    try:
        front = compute_pareto_front(vectors)
    except (ValueError, KeyError, TypeError) as e:
        print(f"error: computing Pareto front: {e}", file=sys.stderr)
        sys.exit(1)
    print(json.dumps(front))


if __name__ == "__main__":
    main()
