# Variant v002 Insight

## Hurt

- **per-op-time**: Latency regressed significantly across all quantiles; caching frequency estimates via additional variable assignments created more overhead than the redundant `sketch.estimate()` call they were meant to replace, suggesting compiler optimization or microarchitectural penalties (increased register pressure, branch misprediction, or cache effects).

## Helped

- **hit-rate**: A marginal improvement (0.7362 → 0.7384) suggests the slight timing changes from refactored control flow had negligible impact on admission decisions, though the gain is within noise margin.

## Takeaway

The optimization to cache frequency estimates during victim scanning backfired, increasing per-operation latency by 42–139% despite theoretically reducing redundant sketch queries. This reveals that algorithmic call-count reduction does not automatically translate to performance gains—the refactored code's additional state management created more overhead than savings. For TinyLFU's latency-sensitive operations, the original structure was either already optimized by the compiler or the microarchitectural cost of the extra variables outweighed the benefit of one fewer hash operation per eviction.
