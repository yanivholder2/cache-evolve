## Recurring wins

- **MRU-end insertion over LRU-end.** Flipping insertion position to the MRU end improved both hit rate and per-op latency simultaneously in cycle 1, suggesting the baseline's default placement was systematically misaligned with the tier hierarchy's warm-up behaviour.
- **Removing dead-weight per-item metadata.** Eliminating an unused timestamp field reduced memory footprint and cut per-operation syscalls at zero hit-rate cost — a dual dividend with no downside observed so far.
- **Orthogonal changes compose cleanly.** Winners in cycle 1 touched unrelated subsystems (promotion mechanics vs. struct layout), implying independently discovered gains are strong candidates for additive stacking.

## Recurring failures

- **Policy-override experiments timing out before scoring.** The LFU eviction override was eliminated by simulator timeout rather than by measured underperformance, so it contributes no useful fitness signal. Experiments that stress the full eviction path under the default trace size are hitting a wall before completion.

## Open questions

- Do the cycle-1 wins (MRU insertion + timestamp removal) remain additive when stacked, or do they interact?
- Are there further per-item struct fields or per-operation syscalls in the baseline that carry no eviction advantage on this workload class, analogous to the timestamp discovery?
- Can the LFU eviction hypothesis be evaluated at all? A lighter trace, smaller cache size, or scoped policy surface might yield a fitness signal before timeout.
- How broadly do these findings generalise across workload traces — are MRU-end insertion gains trace-specific or structural?

## Per-policy notes

- **LRU (baseline default):** Performed sub-optimally with LRU-end insertion; switching to MRU-end insertion recovered significant warm-up efficiency, indicating the default LRU placement logic is a known weak point worth continuing to probe.
- **LFU (forced override):** Untested due to operational timeout in cycle 1. Merit is genuinely unknown; retry recommended under a reduced configuration before the hypothesis is discarded.
- **General:** Metadata overhead (timestamps, auxiliary fields) appears orthogonal to eviction policy choice — gains from pruning are expected to transfer across policy variants.
