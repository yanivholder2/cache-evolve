# Variant v002 Insight

## Helped

- hit-rate: The gain is real but tiny, close to noise. On eviction, the cache now picks the slot with the fewest hits among slots 0–3 instead of always picking slot 0. That changes the outcome only in one case: the item in slot 0 has some hits and a zero-hit item sits elsewhere in the window. Here that case is uncommon. Any hit already sends an item to the MRU tail (index 15). So a hit item only gets back into the window after drifting down about 12 positions with no further hit, and by then it is nearly cold. Saving it from eviction gains almost nothing. Usually the zero-hit item that dies in its place is the newest newcomer, which sits at index 3.

## Hurt

(None measured. This evaluation recorded only hit-rate. The diff reuses the existing `Item::lfu_counter` field, so metadata size cannot have changed. But every miss in a full bucket now scans up to 4 slots, and it may also run an aging pass that halves the counters in the window. Every hit also increments a counter. So a small per-op-time regression is likely but not confirmed.)

## Takeaway

When a hit promotes an item straight to the MRU tail, hit counts at the eviction end add almost no signal on workload-a. With 1/4-depth insertion, recency and the newcomer filter already separate reused keys from one-off keys. The aging rule also halves counters whenever a zero-hit item is evicted next to items that have hits, which happens often. So counters rarely build up enough to matter. If frequency is to add value on top of deep insertion, it should probably act on the promotion side instead of only choosing between the bottom 4 slots. Options include promoting by count, or inserting returning or high-count keys at different depths. Another option is to drop frequency and spend the effort on finding the best insertion depth.
