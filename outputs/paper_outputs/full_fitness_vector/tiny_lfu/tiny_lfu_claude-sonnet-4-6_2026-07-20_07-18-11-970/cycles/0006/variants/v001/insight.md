# Variant v001 Insight

## Helped

- `hit-rate`: The 1-in-8 best-of-2 victim selection occasionally rescues a more-accessed node from eviction by comparing the LRU head against its neighbour, producing a small net reduction in evictions of still-hot items.

## Hurt

- `per-op-time`: Both get and put p50 doubled relative to the parent, and put p99 doubled as well; the `evict_counter: u32` field grew `TinyLfuCache` from 152 to 160 bytes — likely shifting hot fields across a cache-line boundary — and the two extra `sketch.estimate` calls on every 8th eviction add a spike that widens the put tail.

## Takeaway

Combining 1-in-8 best-of-2 victim selection with the bijective-remix base did not produce the additive hit-rate gain the hypothesis expected: the improvement is negligible compared to the parent. More critically, the additional struct field — not the occasional extra sketch probes — appears to be the dominant cost, producing a latency regression across both the get and put paths even though only the put path carries the new logic. This suggests that on this workload the 1-in-8 amortisation rate is not sufficient to offset the structural overhead when the struct size crosses the relevant cache-line boundary.
