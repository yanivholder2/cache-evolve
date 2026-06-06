# Variant v001 Insight

## Hurt

- **per-op-time (put)**: Relaxing the ghost-queue admission threshold from `freq > 0` to `freq >= -1` reintroduced search overhead in the ghost hit detection loop. By accepting freq=0 items as ghost_freq=-1, the ghost queue grew larger with lower-quality candidates, raising the cost of ghost lookups on every put. Put p50 degraded by 378ns (581ns vs 203ns) and p99 by 3405ns (3836ns vs 431ns).

- **hit-rate**: Hit-rate remained virtually flat (0.7159 vs 0.7161), showing that freq=0 items retained in the ghost queue under the relaxed threshold do not provide sufficient re-admission value to offset their search cost in this workload. The items the parent variant excluded proved to be genuine noise, not hidden re-access opportunities.

- **per-op-time (get)**: Get latency degraded marginally (p50: 136ns → 137ns, +1ns), likely due to subtle cache effects from the larger ghost queue structure.

## Takeaway

The hypothesis to recover lost hit-rate by relaxing the ghost-queue filter from pure high-frequency items (`freq > 0`) to marginal candidates (`freq >= -1`) failed catastrophically. Rather than trading latency for hit-rate recovery, the variant sacrificed latency gains without recovering any hit-rate. The parent variant's aggressive signal purity—excluding freq=0 items entirely—was correct: the ghost queue's value lies in tracking proven-valuable access patterns, not in preserving marginal candidates that clutter the search space. The boundary test reveals that further filtering, not relaxation, is the path forward.
