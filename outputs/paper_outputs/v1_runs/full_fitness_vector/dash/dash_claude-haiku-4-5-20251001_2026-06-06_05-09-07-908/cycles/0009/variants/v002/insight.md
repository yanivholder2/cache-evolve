# Variant v002 Insight

## Helped

- per-op-time: Switching from ClassicLRU's vector-shuffling on every access to in-place counter updates and timestamp tracking yields 57% faster get operations; the composite tuple scoring during eviction avoids expensive list reorganization.
- per-op-time: Put latency also benefits from eliminating vector movement, improving by 18% at p50, though the gain is smaller than get since puts don't trigger position updates in the same way.

## Hurt

(none)

## Takeaway

FrequencyAwareLRU succeeds as an implementation optimization: by tracking frequency in-place rather than moving items on access, it cuts per-operation latency substantially. However, the hypothesis's goal of improving hit-rate through frequency-aware eviction does not materialize—the metric holds steady at 0.6634. This suggests that on workload-a, recency and frequency are well-correlated enough that the added complexity of frequency tracking provides no cache decision advantage beyond what pure LRU already achieves. The variant reveals the latency-hit-rate tradeoff: cheaper operations come without hit-rate gain on this access pattern.
