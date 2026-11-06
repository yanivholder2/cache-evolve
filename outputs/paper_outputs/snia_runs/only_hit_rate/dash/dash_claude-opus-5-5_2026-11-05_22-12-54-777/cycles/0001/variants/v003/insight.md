# Variant v003 Insight

## Helped

- hit-rate: Choosing the lowest-count victim from the 8 least-recent slots (ties go to the older slot) lets never-hit scan blocks, which arrive from `put` with counter 0, get evicted before reused blocks that are just as old. Hit-rate rose on both MSR traces at both cache sizes, and hm_0 gained more than web_0. Counting stash hits in `promotion_get` is what makes this work in this segment design. Without it, every item promoted from stash would land at target's front (LRU end) with counter 0 and look the same as a scan block. With it, the item keeps a non-zero count when it is promoted to target and when it is demoted back to stash.

## Hurt

None measured. v003's fitness vector reports only `hit-rate`. It has no `metadata-size` or `per-op-time` entries, so this run neither confirms nor rules out a cost on those dimensions. Two places could add cost. On eviction there is now an 8-slot scan. When a counter saturates, every counter in the bucket is halved, which is O(bucket) work on that hit. `metadata-size` should not move, because the change reuses the existing `lfu_counter` field and leaves `Item` unchanged.

## Takeaway

A small frequency filter over the LRU tail, inside each bucket and with the trait signatures unchanged, beats pure recency on these scan-heavy MSR traces. The gain depends on the counter following an item between tiers, stash hits included. Otherwise the segment's front-insert promotion wipes out the frequency signal. Next variants should measure `per-op-time` before building on this. They could also try a different window size or ageing threshold, since this run only tested 8 slots and a counter ceiling of 15.
