# Variant v002 Insight

## Helped

- metadata-size: Swapping the `Option<usize>` prev/next links for `u32` indices with a `u32::MAX` (`NIL`) sentinel took away both the 8-byte width of each link and the discriminant padding that `Option` adds. `Node<u64,u64>` now holds just key + value + two 4-byte links, the halving the hypothesis predicted (the `node_is_compact` test now asserts it).
- metadata-size (secondary entry): `TinyLfuCache<u64,u64>` also shrank, because `head` and `tail` are now bare `u32`s instead of `Option<usize>`. The sketch's heap size is unchanged, as intended.

## Hurt

- per-op-time: Both get and put regressed at p50 and p99, but the diff doesn't give a credible cause. The hot paths do the same loads and branches as v003. `idx as usize` is a free zero-extension, and `== NIL` compares replace the `Option` matches. The map value shrinking from `usize` to `u32` doesn't shrink the `(u64, u32)` bucket, because padding keeps it at 16 bytes. The sketch and hashing code are byte-for-byte the same as v003. A regression this large and this even across ops and percentiles looks more like a change in the benchmark environment than an effect of the code. It should be re-measured before anyone blames the u32 links.

## Takeaway

Packing the intrusive-list links into `u32` with a sentinel is a pure layout win: it halves the per-entry `Node` and leaves admission and LRU behaviour identical, so hit-rate doesn't move. The cost is a `capacity < u32::MAX` limit, enforced in `try_new`. The per-op-time regression doesn't follow from any instruction-level change in this diff, so re-benchmark v002 next to v003 on the same machine before treating compact links as a speed trade-off.
