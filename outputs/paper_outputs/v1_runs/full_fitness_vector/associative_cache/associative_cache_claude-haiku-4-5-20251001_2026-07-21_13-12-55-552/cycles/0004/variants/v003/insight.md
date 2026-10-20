# Variant v003 Insight

## Helped

- **per-op-time**: Early-exit bucket scanning eliminated unnecessary memory accesses—get operations now return immediately upon finding a key rather than scanning the entire bucket, reducing get p99 latency by 50% from 200 ns to 100 ns. Cache-line alignment (64-byte boundaries) improved spatial locality for the remaining iterations, reducing cache misses and further lowering put operation latencies by 33–50%.

## Hurt

(none)

## Takeaway

The optimization strategy of combining early-exit on key match with cache-line-aligned bucket layouts successfully addressed the tail-latency crisis that plagued the wider 32×64 geometry in the parent variant. By eliminating unnecessary iterations and improving CPU cache efficiency, this variant demonstrates that wide-bucket designs are viable when augmented with latency-conscious micro-optimizations—the fundamental geometry (32×64) and hit-rate gain (0.664) are preserved while tail latencies plummet, validating that the previous cycle's performance trade-off can be resolved through better implementation rather than reverting to narrower buckets.
