# Variant v002 Insight

## Helped

- **per-op-time**: Adaptive divisor formula reduces get latency dramatically (p50: 183ns → 78ns, p99: 376ns → 210ns), likely by enabling cleaner threshold computation that the compiler can optimize or by improving memory access patterns during operation dispatch.

## Hurt

(None observed)

## Takeaway

Implementing a geometry-aware adaptive divisor formula decouples operation latency from hit rate, delivering substantial latency wins (57% on p50 get) while holding hit rate flat at 0.6634. This teaches that threshold adaptation based on allocation ratio is an orthogonal optimization axis: it refines the *efficiency* of promotion logic without changing its *selectivity*. The stable hit rate across the baseline and v002 suggests the current eviction policy already maximizes cache utility for the workload; further gains lie in reducing the computational or memory cost of decision-making, not reshaping what gets promoted.
