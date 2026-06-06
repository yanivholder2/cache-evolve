# Variant v003 Insight

## Helped

- **per-op-time (put p99)**: Tail latency improved 7% (177→165 ns), possibly from reduced variance in saturation-checking logic during bulk promotion flushing, though this gain is marginal and not sustained across other metrics.

## Hurt

- **hit-rate**: Dropped 9.4% (0.6904→0.6255) because deferred promotions in low-saturation mode break LRU ordering; without promotion feedback during get(), eviction reverts to pure FIFO (head-node selection), ignoring access patterns and allowing hot items to drift toward LRU and be evicted prematurely.
- **per-op-time (get p50)**: Median latency increased 71% (34→58 ns) because items are not promoted on access in low-saturation mode, causing hot items to remain at the head of the list and be evicted sooner, increasing cache miss rates and forcing downstream get() misses.
- **per-op-time (get p99)**: Tail latency increased 43% (97→139 ns) for the same reason—the lack of promotion-based feedback degrades the cache's ability to retain frequently accessed items, amplifying both hit rate loss and latency variance.
- **metadata-size (LruCache struct)**: Increased from 192 to 216 bytes due to the addition of saturation-threshold tracking, pending_promotions HashSet, and frequency_buckets HashMap, even though the test workload never saturates enough to use these structures meaningfully.

## Takeaway

Saturation-aware mode switching fails when the activation threshold is never reached. The hypothesis assumed the workload would regularly hit 70% saturation to activate high-saturation mode, but this trace remains light throughout. In practice, the cache stays in low-saturation mode—where deferred promotions eliminate the promotion feedback that LRU relies on. Without promoting items on access, the eviction strategy falls back to insertion-order (FIFO), which is strictly inferior to LRU for temporal-locality workloads. This variant teaches that **deferring correctness-critical operations (promotions) based on unmet saturation assumptions incurs severe latency and hit-rate penalties**, and that conditional logic must either activate universally or preserve baseline semantics when inactive.
