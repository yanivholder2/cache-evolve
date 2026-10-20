# Variant v003 Insight

## Helped

- **hit-rate**: Resurrection count tracking lets previously-evicted keys that re-appear be kept longer in cache by preferring to evict keys with no re-insertion history, giving second chances to working-set boundary items.

## Hurt

- **metadata-size**: The new `eviction_history` HashMap and `eviction_count` field add struct overhead; the HashMap itself consumes ~56 additional bytes per cache instance.
- **per-op-time (get)**: Increased tail latency (p99 spiked from 83ns to 230ns), likely due to worst-case interactions with the eviction_history HashMap or reduced cache locality from the larger metadata footprint.
- **per-op-time (put)**: The eviction scan that looks for a node with resurrection_count == 0 can traverse the LRU list in O(n) time when many nodes have non-zero counts, causing the p99 put latency to nearly double (135ns to 268ns). Median put improved slightly (75ns to 61ns), suggesting most puts avoid the expensive scan.

## Takeaway

Resurrection count tracking successfully improves hit rate by protecting items that thrash at working-set boundaries, but the cost is higher latency variance and increased memory overhead. The improvement is meaningful (~4% hit gain), but the approach incurs O(n) eviction scans and HashMap overhead that hurt tail performance, making it a trade-off favoring throughput over latency predictability.
