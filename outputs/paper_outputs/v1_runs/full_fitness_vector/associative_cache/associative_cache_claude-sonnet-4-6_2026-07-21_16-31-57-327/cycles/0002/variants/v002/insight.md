# Variant v002 Insight

## Helped

*(no dimensions improved)*

## Hurt

- **hit-rate**: Swapping the tail item into the evicted slot breaks strict relative ordering of unaccessed items, causing the eviction candidate (always index 0) to sometimes be a more recently accessed item than it would have been under `remove+push`; this degrades hit quality on the Zipfian workload.
- **per-op-time**: Despite replacing an O(n) `Vec::remove` with an O(1) `swap`, get p50 latency increased rather than decreased — indicating that at the small bucket sizes this N-way set uses, the bounded linear shift is not the latency bottleneck, and the reordering introduced by swap may worsen element locality on subsequent accesses within the same bucket.

## Takeaway

At the small, bounded associativity degree of this cache, `Vec::remove`'s element shift is cheap enough that it does not dominate hit-path latency, so the O(1) swap provides no throughput gain. Worse, the approximated ordering it produces actively hurts hit rate, making this change a loss on both competitive dimensions simultaneously. Future latency optimizations should look outside the ordering step — e.g., the bucket hash dispatch or value copy cost — rather than assuming the linear shift is the hot path.
