# Variant v003 Insight

## Helped

- **per-op-time (put)**: Deferring main-queue eviction until 90% occupancy reduces eviction overhead, significantly speeding up put operations that would otherwise trigger expensive frequency-decrement loops.

## Hurt

- **per-op-time (get)**: Deferred evictions leave the main queue more populated, increasing the search space for linear lookups during get operations and degrading lookup latency.

## Takeaway

Lazy eviction trades get latency for put efficiency, but fails to recover hit-rate as hypothesized. The deferred evictions keep more items in the main queue longer, which speeds up writes by reducing eviction calls—however, fuller queues slow reads through longer linear scans. The lack of hit-rate improvement reveals that v001's precision bottleneck is the admission filter (strict promotion threshold), not eviction frequency; deferring main-queue operations doesn't address the core constraint on item residence time.
