"""Hit-rate fitness plugin.

Receives a pre-computed `CaffeineResults` (one entry per (trace, size)
point) and emits the plugin JSON contract. Issue #85 produces exactly
one entry; #83 will extend to a real trace suite.
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
    summary = sum(v["hit_rate"] for v in values) / len(values)
    return {
        "dim": "hit-rate",
        "values": values,
        "summary": round(summary, 6),
        "higher_is_better": True,
    }
