# Variant v002 Insight

## Helped

(none)

## Hurt

- hit-rate: Pure FIFO eviction with selective window-based promotion cannot preserve hot items as effectively as frequency-aware LRU, losing 4.8% hit rate despite the window strategy.
- metadata-size: The HashSet for deferred_promotions and window_size tracking increased LruCache overhead from 112B to 168B, offsetting any per-node savings from removing frequency counters.
- per-op-time: The window checking mechanism (is_in_window) traverses backward from tail in O(window_size) time per get, plus HashSet operations on deferred_promotions queue, causing p99 get latency to explode from 83ns to 252ns (+203%).

## Takeaway

Decoupling batched queue-flush from frequency tracking does not yield orthogonal algorithmic gains. The batching optimization that succeeded in cycle/0007/v001 was not separable from the frequency-bucketing context—it relied on better eviction decisions to justify the batching overhead. In a pure FIFO context with window-limited promotion, the added complexity of deferred queues and window checks introduces more latency cost than the simplified eviction decision provides, resulting in degraded hit rate and significantly worse tail latencies across all operations.
