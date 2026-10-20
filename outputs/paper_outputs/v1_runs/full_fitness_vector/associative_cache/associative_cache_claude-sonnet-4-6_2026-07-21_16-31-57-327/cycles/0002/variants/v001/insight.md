# Variant v001 Insight

## Helped

- hit-rate: Replacing recency-based eviction with a frequency-sorted order protects against scan pollution on the LIRS trace — infrequently-accessed scan items accumulate at the tail and are displaced before they can crowd out hot items.

## Hurt

- per-op-time: Every `get` now increments `lfu_counter` and enters the bubble loop (even when the item is already at position 0), adding unconditional overhead that raised get latency above baseline; put latency improved (tail removal needs no shifting), but the summary metric is driven by get, so the net result is slower.

## Takeaway

Maintaining a descending-frequency invariant via bubble-sort on access successfully delivers the hit-rate benefit of LFU while keeping eviction O(1), but the per-access counter increment and loop entry on `get` impose a fixed overhead that the baseline's simpler recency path did not pay — the O(1) eviction win on `put` cannot compensate for this in the p50 summary. Future variants should look for ways to amortize or skip the bubble step when the accessed item's relative rank is already stable (e.g., a guard comparing the counter to its left neighbour before entering the loop).
