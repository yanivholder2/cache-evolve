# Variant v003 Insight

## Helped

- **per-op-time (get p99)**: Tail latency dropped 34% (147→97 ns) by using simple bounded-window eviction in default mode instead of always paying frequency-bucket pointer-chasing overhead; v002's bucket promotion logic required unlinking and relinking nodes, which dominated worst-case latency.
- **per-op-time (put p99)**: Tail latency improved 7% (190→177 ns) for the same reason—simple mode avoids the bucket reorganization cost on normal insertions.
- **per-op-time (get p50)**: Median latency improved 3% (35→34 ns) by eliminating the constant per-operation bucket management overhead when running in simple mode.

## Hurt

- **metadata-size**: LruCache struct size increased 60% (120→192 bytes) due to maintaining dual-mode state: saturation tracking (saturation_threshold, hit_rate_threshold, recent_hits, recent_accesses), bucketing flag, and simple-mode list pointers (head, tail). The cache now carries both bucket infrastructure and simple-LRU pointers simultaneously, even though only one is active.
- **per-op-time (put p50)**: Median put latency increased 3% (70→72 ns) because every put now calls should_use_bucketing() and update_hit_rate() to monitor mode-transition triggers, adding a constant overhead regardless of which mode is active.

## Takeaway

Adaptive bucketing successfully trades fixed memory overhead for variable-latency reduction. On this underutilized workload (which never triggered high-saturation mode), the cache spent most operations in simple mode, cutting p99 latency by ~34% for gets and 7% for puts. The hit rate remained stable (0.6904) because bounded-window scan is sufficient for eviction decisions on this trace. However, the cost is permanent: the LruCache struct now carries 72 extra bytes to maintain state for mode transitions, even on caches that never saturate. This design is profitable for latency-sensitive workloads with variable utilization patterns, but the memory footprint tax applies universally.
