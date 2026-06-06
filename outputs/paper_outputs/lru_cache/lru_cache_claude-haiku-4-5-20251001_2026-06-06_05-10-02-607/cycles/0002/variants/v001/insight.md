# Variant v001 Insight

## Helped

- **per-op-time (put p99)**: Bounded search window eliminates pathological O(N) scans; tail latency dropped from 2965ns to 927ns by scanning at most min(capacity/4, 256) nodes before falling back to LRU eviction, preventing worst-case behavior when most items accumulate high frequency.
- **per-op-time (get p50)**: Minor improvement from 36ns to 35ns, likely due to subtly reduced contention during eviction when the search completes faster.
- **per-op-time (get p99)**: Minor improvement from 67ns to 64ns, consistent with reduced GC/eviction pressure on the critical path.

## Hurt

(none)

## Takeaway

The bounded search strategy successfully addresses the pathological O(N) eviction latency spike observed in v002. By limiting the eviction scan to the first `min(capacity/4, 256)` nodes, the variant guarantees worst-case eviction time while preserving hit rate (0.6721 unchanged) and the frequency-aware eviction benefits. The dramatic improvement in put p99 latency (3.2× faster) demonstrates that this approach is effective for workloads where items accumulate high frequency during steady-state operation—trading off the theoretical ideal of finding the globally best eviction target for predictable, low tail latency.
