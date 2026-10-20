# Variant v002 Insight

## Helped

(None)

## Hurt

- **hit-rate**: Segment-based eviction selects the oldest-containing segment then its LRU item, which diverges from true global LRU and produces suboptimal eviction choices, increasing misses.
- **per-op-time**: Maintaining segment metadata via `update_segment_metas()` on every access and modification adds O(bucket_size) overhead to all operations, overwhelming the reduction in rare eviction scan depth.

## Takeaway

Segmenting buckets into 8 fixed regions successfully bounds eviction scans to 8 items instead of 64, but the constant per-operation cost of updating segment metadata on every get, put, and evict negates this gain. The design optimizes an infrequent code path (eviction) at the expense of high-frequency paths (all cache operations), causing p99 latencies to nearly 4× and hit-rate to drop by 6.2%. The heuristic of selecting the segment with the oldest item, then evicting within that segment, is also a weaker eviction policy than pure global LRU, compounding the hit-rate loss.
