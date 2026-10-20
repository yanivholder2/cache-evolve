# Variant v001 Insight

## Helped

- per-op-time (get p50, the summary): `find_tagged` now scans a parallel `Vec<u16>` of tags in 16-wide blocks and reads a full key only where a tag matches. Misses no longer walk the whole bucket at the item stride, and the hash used for the bucket index is reused for the tag instead of being computed twice. The get median improved even though each `Item` in this build is wider than in v003, which makes the tag scan look like a real win rather than a side effect of layout.

## Hurt

- metadata-size: The overlay did not change `Item`, yet `Item<u64,u64>` regressed to the size it had before cycle 3. The likely cause is that v003's overlay carried the 24-byte `src/shared/item.rs` from cycle 3 (a `u32` tick and a `u32` LFU counter), while v001's overlay has no `item.rs` at all. The build probably fell back to the base `Item` with an `Instant` timestamp and a `usize` counter. The hypothesis expected this size to stay the same.
- per-op-time (put p50/p99): The put path gave back the saving that removing the second hash was meant to buy. I think the main cause is the same lost `item.rs`. Every `Item::new` on put would read the wall clock instead of bumping a thread-local tick. Every `insert`, `rotate_left`, and `remove` would also move wider items, on top of the new per-move work on the parallel `tags` vector (`tags.insert`, `tags.rotate_left`, `tags.remove`) that every branch of `pair_put_tagged` now does.

## Takeaway

On the get path, the change works. A 2-byte tag array, scanned forward in auto-vectorisable blocks and fed by a single hash, cuts the get median without changing hit rate, because exact-key confirmation leaves SLRU and ghost behaviour bit-for-bit the same as v003. The regressions in metadata-size and put look like a lost carry-forward of v003's 24-byte `item.rs`, not a cost of tagging. The next variant should restore that file on top of this one before judging how much the parallel-tag bookkeeping really costs on put.
