# Variant v003 Insight

## Helped

- hit-rate: Choosing between a len/4 and len/2 insertion slot from a global hit/miss counter, plus one-step transpose promotion on hit, keeps reused entries from being flushed by scan bursts. That beats pure LRU on the LIRS workload.
- per-op-time: Put latency, both median and tail, dropped clearly. Swap-based insertion at a computed index avoids LRU's full-bucket shift/reordering on every put. The get tail also tightened.

## Hurt

- per-op-time: Median get latency is marginally higher. The extra saturating-counter update on every get, plus the transpose swap on hit, adds a small fixed cost to the hottest path, even though it is branch-free.

## Takeaway

The adaptive counter fits in existing padding, so metadata did not grow, and the combination of mid-bucket insertion, transpose promotion and swap insertion gave a better hit rate and much cheaper puts. The only cost was a slight increase in median get time. Because this variant also changes the promotion and insertion mechanics relative to LRU, the result cannot show how much of the gain comes from the adaptive len/4-vs-len/2 switch itself. Isolating that would need a fixed-position comparison.
