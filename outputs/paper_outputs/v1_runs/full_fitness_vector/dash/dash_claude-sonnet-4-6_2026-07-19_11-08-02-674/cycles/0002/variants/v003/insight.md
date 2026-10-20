# Variant v003 Insight

## Helped

- **hit-rate**: Bypassing a saturated target on stash hits stopped the cascading eviction that displaced a warm resident for every returning stash item, so the target's resident set survives longer and converts more future accesses into hits.

## Hurt

- **per-op-time**: The added `is_full()` branch on every stash hit, combined with routing through `promote_to_probing` (which may itself trigger a demotion cascade when probing is full), raised p99 latency for both get and put operations, even though the p50 median was unaffected.

## Takeaway

Skipping the target when it is already full eliminates one unnecessary eviction per stash-hit and meaningfully improves hit rate, confirming that unconditional stash→target promotion was actively harmful under saturation. The trade-off is a higher tail latency on the stash-hit path: the conditional check and the probing-insert cascade add work that only shows up in the p99, suggesting this change benefits workloads where hit-rate gains outweigh occasional worst-case get/put latency spikes.
