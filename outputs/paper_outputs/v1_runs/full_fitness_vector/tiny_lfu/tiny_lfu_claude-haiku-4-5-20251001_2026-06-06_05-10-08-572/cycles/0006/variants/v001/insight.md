# Variant v001 Insight

## Helped

- **per-op-time**: Reducing the victim candidate pool from N=5 to N=2 cuts scanning overhead during eviction, improving put operation tail latency (p99) by 30% (570ns → 400ns) while keeping median latency nearly flat.

## Hurt

- **hit-rate**: Narrowing to N=2 causes a negligible hit-rate decrease (0.7355 → 0.7354), confirming that checking only two candidates occasionally misses the frequency optimal victim, though the effect is minimal.

## Takeaway

This variant validates the latency-frequency tradeoff frontier: N=2 provides substantial tail-latency recovery (30% reduction in put p99) at negligible hit-rate cost, confirming that examining the two coldest items captures most frequency-awareness benefits. The minimal hit-rate drop indicates N=2 sits at the practical lower bound of effective frequency-aware selection—the head and next item in the LRU list suffice to identify a frequency minimum in most cases, but deeper scanning adds no further value. For latency-critical deployments, N=2 offers a strong compromise; workloads prioritizing hit-rate should consider N=3 or N=4 to recover the marginal frequency-awareness advantage.
