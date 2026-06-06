# Variant v002 Insight

## Helped

- **hit-rate**: Hybrid frequency-LRU eviction strategy improved hit rate by retaining frequently-accessed items instead of always evicting the oldest; frequency tracking allows the cache to skip over popular items during eviction even if they are older.
- **per-op-time (get)**: Both p50 (41→36 ns) and p99 (83→67 ns) latencies improved, likely due to better spatial locality or reduced contention when accessing high-frequency items that remain in cache longer.
- **per-op-time (put p50)**: Typical put operations are 16% faster (75→63 ns), benefiting from the hybrid strategy avoiding churn of frequently-accessed items.

## Hurt

- **metadata-size**: Each Node expanded from 48 to 56 bytes (+17% per-item overhead) for the new `freq` field, plus 8 additional bytes in LruCache for eviction tracking; this is paid even on cache misses and underutilized caches.
- **per-op-time (put p99)**: Tail latency spiked dramatically from 135 ns to 2965 ns (+20×) when evicting under contention, because `find_evictable_node()` must scan from head through many high-frequency nodes to find one with freq≤1; pathological access patterns where most items have high frequency trigger a full linear scan.

## Takeaway

The hybrid frequency-LRU approach successfully improves hit rate by preferring to evict cold items. However, it trades away put tail latency for that gain: when cache entries accumulate high frequency (a realistic steady-state), the eviction scan degrades to O(N) search through high-frequency nodes. The frequency decay mechanism helps over time, but cannot prevent worst-case p99 behavior on sudden workload shifts. The design prioritizes reducing cache misses over consistent latency, suitable for throughput-sensitive workloads that tolerate occasional slow puts.
