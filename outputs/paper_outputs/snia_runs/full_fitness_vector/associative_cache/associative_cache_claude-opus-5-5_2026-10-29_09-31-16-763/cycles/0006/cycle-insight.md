# Cycle 0006 Insight

## Cohort outcome

All three variants branched from cycle 0005/v002 (the byte-tag base). v001 merged in the per-bucket clock and dropped timestamp stores on ClassicLRU. v002 fused the full-bucket evict and quarter-depth insert into one partial rotation. v003 added a 1-bit second-chance reuse marker at the LRU end. v002 is the only clear win. v001 held every invariant but its per-op time got worse. v003 is a mixed policy result.

## Patterns

- The fused rotation is now confirmed as a put-path win, both p50 and p99, with hit rate exactly unchanged. Measured alone, it shows the two memmoves were the main cost of a miss on a full bucket. What remains on that path is a quarter-length shift plus the tag mirror.
- Removing the atomic clock did not stack on top of the tag base. v001 only removes work, yet get and put both slowed. That points to measurement drift or to the bucket's stride growing from the extra `clock` field, not to the logic. Cycle 0005's clock gain is real on the old base, but on the tag base the uncontended atomic was already cheap. Bucket layout now looks like a possible hidden cost.
- Get p99 rose in both v001 and v002, even though neither touches the get path. Get-tail movement in this cohort is most likely noise or code layout, so don't attribute it to a diff without a same-session rerun.
- A reuse bit that is set on every hit and cleared only at eviction saturates on large caches. Once that happens, second chance becomes reordering near the LRU end with no real signal behind it. It helped only the small hm_0 cell, cost some web_0 scan resistance, and its rotations hurt the put tail. This is the fourth reuse signal to fail the two-workload tradeoff. It also confirms cycle 0005's rule: a probationary bit needs its own decay.
- Every hit-rate-neutral rewrite again matched all four cells exactly. Policy semantics stay stable under the mechanical speed changes.

## Open questions

- Does v001's slowdown hold up in a back-to-back same-session rerun? If it does, does folding `clock` and `bip_tick` into one shared `u32`, so the bucket stays the parent's size, remove it?
- Can the remaining quarter-length shift be cut further, for example with a ring offset for the probationary region, or by keeping tags-only movement on the hot path?
- Would a reuse bit with independent decay (cleared on a sweep or when the item passes quarter depth), with spared items placed outside the admission region, keep v003's small-cache gain without the large-cache saturation?

## Continuation

Use v002's fused rotation as the new base. Re-apply v001's clock removal on top only if the bucket size stays unchanged (one shared `u32` for `clock`/`bip_tick`), and judge it against a same-session parent baseline. Keep attacking the residual quarter-shift cost on put. Drop v003-style second chance as it was built. Revisit it only with explicit independent decay and placement outside the admission region. Otherwise keep the policy frozen.
