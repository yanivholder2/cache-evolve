# Variant v002 Insight

## Helped

- **hit-rate**: Frequency bucket organization preserves the frequency-aware eviction strategy that was introduced in the parent variant, achieving slightly better hit rate (0.6721 → 0.6904) by reliably evicting low-frequency items.
- **per-op-time (put p99)**: Tail latency improved dramatically from 2965 ns to 190 ns by eliminating the O(N) linear scan bottleneck in `find_evictable_node()`; direct bucket indexing finds eviction candidates in constant time regardless of frequency distribution.

## Hurt

- **per-op-time (get p99)**: Tail latency increased from 67 ns to 147 ns because promoting a node to a higher frequency bucket requires unlinking from its current bucket and re-linking, adding pointer-chasing overhead compared to the simple LRU list mutation.
- **per-op-time (get p50)**: Typical get latency increased slightly from 36 ns to 35 ns—essentially unchanged, though bucket management adds minor overhead that occasionally surfaces in measurement noise.
- **per-op-time (put p50)**: Typical put latency increased from 63 ns to 70 ns due to the cost of inserting new nodes into frequency buckets and updating min_freq tracking, versus simple tail-append in the old LRU list.

## Takeaway

Frequency bucket organization successfully trades operational latency variance for catastrophic tail latency elimination. The parent design's O(N) eviction scan created a 45× p99 spike (135→2965 ns) in pathological cases; reorganizing into buckets eliminates that worst-case, dropping p99 from 2965 ns to 190 ns at the cost of making normal-case operations 5–10% slower. Hit rate remains stable, showing the frequency-aware strategy is sound. This design is suitable for latency-sensitive workloads where predictable p99 bounds matter more than minimizing median latency, and where the small per-operation slowdown is acceptable trade-off.
