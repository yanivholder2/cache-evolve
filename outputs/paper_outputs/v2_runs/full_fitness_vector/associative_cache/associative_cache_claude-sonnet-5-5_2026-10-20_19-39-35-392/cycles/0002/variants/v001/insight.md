# Variant v001 Insight

## Helped

- hit-rate: Second-chance eviction keeps the recency signal through a reference bit, and it ignores one-off scans. Reused entries survive eviction in the full bucket, so hit rate rose slightly over strict LRU.
- per-op-time (put): `put` no longer shuffles items to keep a recency order. It does a short clock-hand scan and overwrites one slot, so median and tail put latency both dropped noticeably.
- per-op-time (get p99): A hit now only sets a bit, with no item movement. That trimmed the get tail.

## Hurt

- per-op-time (get p50): The median get time is about the same as baseline, slightly higher. The hit path was already cheap, and the bit-set plus slot lookup adds nothing measurable. This supports the cycle 0001 finding that item movement is not what dominates get time.

## Takeaway

Swapping move-to-front LRU for a CLOCK reference bit gave a small hit-rate gain. Its main payoff was cheaper, tighter `put` (and get-tail) latency, because the cost of maintaining LRU order sat mostly on the write and eviction path. Metadata stayed at the baseline size, so the policy change cost no memory. Median `get` is not improved by a cheaper hit path, which points to the lookup itself as the remaining bottleneck.
