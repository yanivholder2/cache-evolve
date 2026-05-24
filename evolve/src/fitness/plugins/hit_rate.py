"""Hit-rate fitness plugin.

Receives a pre-computed `CaffeineResults` (one entry per (trace, size)
point) and emits the plugin JSON contract.
"""

from __future__ import annotations

from src.fitness.caffeine_simulator import CaffeineResults


def compute(*, results: CaffeineResults) -> dict:
    if not results.results:
        raise RuntimeError("hit-rate plugin: no CaffeineResults to summarise")
    values = [
        {"trace": r.trace, "size": r.size, "hit_rate": r.hit_rate}
        for r in results.results
    ]
    summary = sum(r.hit_rate for r in results.results) / len(results.results)
    return {
        "dim": "hit-rate",
        "values": values,
        "summary": round(summary, 6),
        "higher_is_better": True,
    }
