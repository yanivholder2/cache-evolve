# Variant v002 Insight

## Helped

- **per-op-time (put)**: Replacing `min_by_key` closures with explicit loops in eviction candidate search reduced overhead; additionally, early-exit for single-item buckets and zero-counter detection in Lfu eliminated unnecessary iterations during eviction, improving put p50 from 143ns to 100ns.

## Hurt

- **per-op-time (get)**: Despite preserving ClassicLRU's logic and leaving the hit-path untouched, get operations degraded from 69ns to 100ns (p50). Structural changes to the eviction_policy module—adding explicit loops and early-exit branches—likely triggered different compiler optimization and inlining decisions that regressed the common path's cache locality or instruction pipeline efficiency.

## Takeaway

The variant successfully optimized the put path by replacing functional iterator patterns with explicit loops and early-exit conditions in eviction candidate discovery. However, the cost was a regression in get latency, suggesting that micro-optimizations to the less-frequent eviction code can inadvertently disrupt the compiler's optimization of the hot path through binary layout or inlining side effects. Early-exit optimization in isolation is viable only if it avoids structural changes to shared code paths.
