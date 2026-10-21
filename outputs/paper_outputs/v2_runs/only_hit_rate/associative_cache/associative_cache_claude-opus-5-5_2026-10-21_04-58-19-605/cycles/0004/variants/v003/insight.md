# Variant v003 Insight

## Helped

- hit-rate: Each bucket now keeps a 4-entry ghost ring of recently evicted keys. A newcomer whose key is in that ring is admitted with GCLOCK counter 1 instead of 0, and its ghost entry is used up. A key that returns soon after eviction therefore survives the hand that rests on it after insertion: the next sweep only decrements it and moves on. Before this change, the one-miss LIP filter evicted such a key at once. Hit rate rose above the parent's figure, which was the threshold H03 set, so the hypothesis held. This confirms that the parent's filter was losing keys whose reuse distance was just over one bucket miss.

## Hurt

(No measured dimension regressed. Only `hit-rate` is in the fitness vector. Two costs are real but were not measured. First, metadata grows by four `Option<K>` slots plus one byte per bucket. Second, every miss on a ClassicLRU bucket now does a linear scan of the ghosts, so per-op time on the miss path may rise slightly. The gain also comes from a single trace at a single size, workload-a at 2048.)

## Takeaway

The parent's aggressive one-miss filter was right for one-shot keys but too strict for keys that come back quickly. The fix was a small, recent ghost ring that grants only partial protection: a counter of 1, which buys one extra sweep pass. This is unlike cycle 0002's MRU-on-ghost-hit, which gave full protection and hurt. Recency evidence from evictions pays off on this CLOCK line only when the promotion it triggers is capped. Two things should be tested next. One is the size of the ring, since 4 is a guess. The other is whether the gain holds on traces with longer reuse distances, where most returning keys would have already dropped out of a 4-deep ring.
