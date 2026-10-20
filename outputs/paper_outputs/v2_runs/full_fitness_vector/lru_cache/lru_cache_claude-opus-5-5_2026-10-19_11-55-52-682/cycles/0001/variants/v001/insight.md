# Variant v001 Insight

## Helped

- hit-rate: Splitting recency into probationary and protected segments means a key needs a second touch before it is protected. On a hit, the probationary node is moved to the protected MRU end. Only the probationary LRU is evicted, so the one-touch scan and loop keys that the LIRS workload-a trace is built from no longer push out reused keys. Under the baseline's single LRU list, those reused keys were flushed.
- metadata-size: Each node had two `Option<usize>` links. They became two `u32` indices that use `u32::MAX` as the "none" value, and the new `seg: u8` tag fits into the padding that this frees up. So the node got smaller instead of growing to hold the segment tag. It ended up well below the 48-byte limit the hypothesis allowed, and the variant's own unit test now checks that it fits in 32 bytes. The cache struct stayed the same size even with four head/tail pointers, `prot_len` and `prot_cap`, because the `u32` heads and tails take half the space of the old `Option<usize>` pair.
- per-op-time: The SLRU logic adds only one branch on the segment tag, plus at most one extra unlink and push_back when the protected segment overflows. Both get and put got faster anyway, at the median and at p99. The smaller nodes are the most likely reason: more of the arena stays in cache, and following a link reads half as many bytes. Put probably also gains because a full cache picks its victim straight from `prob_head`.

## Hurt

None of the measured dimensions got worse. The costs are outside what we measure. Capacity is now capped at `u32::MAX - 1`, and `try_new` rejects anything larger. Eviction order is no longer strict LRU, so the in-file tests that assumed LRU order had to be rewritten.

## Takeaway

Using SLRU with an 80% protected cap is an effective, cheap defense against scans on the LIRS trace. Requiring a second touch before a key is protected is enough to beat plain LRU's hit rate here. The more important lesson is about layout: switching links from `Option<usize>` to `u32` paid for the segment tag and also made the node smaller. That is very likely why speed and footprint improved as well, which means the layout change could improve any later list-based policy, separate from the eviction policy itself.
