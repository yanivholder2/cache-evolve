# Variant v001 Insight

## Helped

- per-op-time (get): Lookups now scan a parallel `Vec<u16>` of tags in 16-wide branch-free blocks and only compare full keys where a tag matches. Misses and probation hits no longer compare 16-byte keys across the whole merged bucket, and `mod.rs` hashes the key once for both the bucket index and the tag. Together these removed most of the get-median slowdown that the 7/8 cap added in the parent. The tail barely moved, which fits a tail driven by the protected-segment `rotate_left` that this change left alone. Tag upkeep also adds a second, 2-byte rotate.
- per-op-time (put): Put also got faster at both the median and the tail, although the hypothesis did not predict it. Put shares the single hash and the tag-first re-find. The ghost-ring and eviction paths now reuse the stored tag (`tags[0]` passed to `record_ghost`, and the passed-in tag in `take_ghost`) instead of rehashing the evicted or incoming key. So put drops more hashes than get does, and that likely outweighs the extra `tags.insert`/`remove`/rotate work.

## Hurt

None. hit-rate and metadata-size held at the parent's values, as predicted, because the tags are a pure lookup accelerator. Replacement order and the 16-byte `Item` are unchanged. Note that metadata-size only measures `Item` and the cache header. The new 2 bytes per slot live in a separate heap `Vec` and do not show up in this dimension.

## Takeaway

The lookup lever and the protected-cap lever are separable. Tag-first `find` plus one hash per op lowers latency on top of the 7/8-cap SLRU without changing any replacement decision, so hit rate is exactly preserved. Keeping tags in lockstep costs a second, narrow rotate, and that cost is clearly smaller than the full-key compares and repeated hashes it removes, including those on the ghost/eviction path in put. What remains of the latency cost is the protected-segment rotate itself, and that tail is what the next change has to target.
