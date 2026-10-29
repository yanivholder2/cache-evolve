# Variant v001 Insight

## Helped

- hit-rate: Not a gain, but the invariant held. All four cells match the parent exactly, as the hypothesis required. The memo only decides whether `put` skips `get_position_tagged`. BIP counting, quarter-depth insertion and the fused rotation are untouched, and every mutating path clears `miss_valid` first, so no resident key is ever inserted twice. The `debug_assert` in `put` and `memo_never_duplicates` both check this.
- metadata-size: Flat. `Item<u64,u64>` and the top-level `AssociativeCache<u64,u64>` handle do not change. The bucket stride did grow from 80 to 88 bytes because of `miss_hash`, but this dimension does not measure it. So the extra memory cost of the memo does not appear here and has to be counted separately.

## Hurt

- per-op-time (put): Put got clearly slower at both p50 and p99, the opposite of the prediction. Every lip `put` now pays for new work: it reads `miss_valid` and `miss_hash`, does a 64-bit compare, and stores `miss_valid = false`. Any memo hit also has to make up for 8 more bytes of bucket stride. The only possible saving is one tag scan, and only when a `get` miss on the same key just happened. In the parent, that scan is a short, branch-free 16-wide mask loop over a dense `u8` array that is already in L1 after the get. The regression is too large to blame on run-to-run noise. So in this benchmark the bookkeeping costs more than the scan, or the memo rarely fires, or both. The eval output here cannot tell those apart.
- per-op-time (get): Get p50 barely moved, but get p99 rose noticeably. This fits the new side effect on the get-miss path: `get_position(&self)` used to be read-only, and now it stores `miss_hash` and `miss_valid` into the bucket on every miss of an exact-hashed key. A lookup now dirties the bucket's cache line, and the larger 88-byte stride makes buckets straddle cache lines more often.

## Takeaway

Caching the result of a "confirmed absent" check, to skip the tag scan in a following `put`, is a bad trade in this design. After cycles 0005–0007, the scan costs less than the memo's per-op loads, stores and compare, plus the extra 8 bytes of stride and the writes that `get` now makes. The idea that removing a whole unit of work pays off only holds when that work costs more than the state needed to skip it, and here it does not. The hypothesis also hoped to answer the open stride question, but this variant changed stride and per-op work together, so it can't. A clean stride test needs to pad the parent to 88 bytes with no behavioural change. Revert the memo and keep `get_position` read-only.
