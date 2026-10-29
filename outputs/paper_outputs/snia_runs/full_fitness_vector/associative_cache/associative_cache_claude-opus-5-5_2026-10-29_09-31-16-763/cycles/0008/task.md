# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0008

## Winners

- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Two speed variants started from cycle 0004's 24-byte `Item`. v001 replaced the global atomic tick with a per-bucket counter. v002 added a dense 1-byte fingerprint pre-filter to the get scan. A third, policy variant (v003) made LIP insertion depth adapt per bucket. All three are Winners. v001 and v002 made clear per-op gains with hit rate exactly unchanged. v003 is the weakest: its hit-rate summary fell, and it only had small gains in a couple of cells.

## Patterns

- Both long-open speed questions are now answered, and both answers are positive. The cycle 0004 regression came from the shared atomic clock, not from the environment or the denser layout. With the atomic gone, the 24-byte `Item` is at least as fast as the old baseline. The fingerprint pre-filter, recommended for three cycles, is the first change that actually reduced full-key compares. It sped up get and put at both p50 and p99.
- Density is what made the side array work. A `u64` key mirror failed in cycle 0002, while a byte-tag stream 24× denser than items, scanned in vectorisable chunks, succeeded. Misses dominate, and every new-key put runs a miss scan, so the miss path is where the savings land. A small byte memmove on hits is an acceptable cost.
- The two speed wins are independent and have not been combined. v002 still carries the atomic tick that v001 removed. Their gains come from separate costs (the shared-line RMW on put, and compare count on scan), so stacking them should give both.
- Hit position inside one LRU-ordered bucket is not a usable reuse signal. Hits below the insertion point mix new admits with old items that have aged down. As a result, adaptive depth read "pressure" and admitted keys deeper, which hurt the hm_0 cell it was meant for and also web_0 scan resistance. This is the third reuse or adaptation signal in a row to fail the two-workload tradeoff. The fixed quarter-depth LIP with BIP 1/32 is still the best policy.
- Metadata-size is still blind to per-bucket state. Tags in v002 and extra bucket fields in v003 both added real footprint, and the dimension reads flat for both.

## Open questions

- Do the per-bucket clock and the fingerprint filter stack additively? Can the timestamp stores be skipped completely when no active policy reads them?
- Can tags be extended beyond the ClassicLRU/LIP path, for example to CLOCK? Would wider tags, or SIMD on the 16-wide mask, cut the remaining false-positive compares?
- Is fused rotation with `ptr::copy` (cycle 0004 v001, never tested cleanly) worth retrying now that the atomic is gone, so the put tail can be measured on its own?

## Continuation

Merge v001 and v002 into one base: the 24-byte `Item`, a per-bucket non-atomic clock (or no stamp at all when no policy reads it), and the byte-tag pre-filter. Then measure the cleaned-up fused rotation on top of that base to attack the remaining put cost from the memmove. Drop adaptive depth based on hit position. Do not attempt more reuse or adaptation signals unless one carries an explicit per-item probationary bit with symmetric grow and decay. Otherwise, keep the policy frozen and spend the next cycles on speed.


### Cycle 0006

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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

All three variants branched from cycle 0006/v002 (fused rotation on the byte-tag base). v001 inlined the ClassicLRU hit path using `rotate_left(1)`. v002 made the BIP period depend on bucket width (1/32 narrow, 1/128 wide). v003 removed the global atomic tick while keeping bucket stride fixed. All three are Winners, but none is a clean speed win. v002 gave the only real movement, a small net hit-rate gain. v003 closed an open question without improving speed. v001 is a negative result that tells us how to do it next time.

## Patterns

- Per-op-time drifted again. Both v002 and v003 got slower on code paths they barely changed or only removed work from, and get-p99 swung in both directions with no code change behind it. Three cycles of this mean single-run tail numbers should not be used to judge anything. Only same-session, repeated runs against the parent count.
- Removing the atomic clock is settled as speed-neutral on the tag/fused base. Bucket stride was pinned (80 bytes) and there was still no put gain. The atomic is uncontended and stays in L1, so it was never the bottleneck. Keep the change for correctness and concurrency only, and stop expecting a speed win from it.
- Generic standard-library helpers can hide slow paths. `rotate_left(1)` on short suffixes falls into a scalar loop that moves one element at a time and cost more than the remove+push memmoves it replaced. The pattern that wins on put is a hand-written `ptr::read`/`ptr::copy`/`ptr::write`, and it hasn't been applied to the hit path yet.
- A static policy split by bucket width was the first hit-rate change in several cycles that did what it predicted. Web_0 on wide buckets improved, and narrow cells stayed bit-identical. But hm_0 on the same wide buckets gave most of it back, so on wide buckets the BIP rate is close to a zero-sum knob between the two workloads, not a free lever.
- Hit-rate invariants again held exactly for every mechanical rewrite. Policy semantics stay robust to restructuring.

## Open questions

- Does an inline, devirtualised hit path with an MRU-skip and a manual `ptr::copy` shift (with the same done for tags) finally lower get p50?
- Is there a wide-bucket BIP period between 1/32 and 1/128 (for example 1/64) that keeps most of the web_0 gain and less of the hm_0 loss? Or is that curve flat?
- Do v002's and v003's apparent slowdowns go away in a back-to-back same-session rerun?

## Continuation

Merge v002's width-keyed BIP and v003's stride-neutral clock removal into one base. Both keep hit rate and size invariant. Re-measure it against cycle 0006/v002 in the same session. On that base, retry v001's hit-path idea with a hand-written `ptr::copy` shift instead of `rotate_left`. Treat the wide-bucket BIP period as a tunable worth at most one more probe, such as 1/64. Drop any further speed expectations from removing the atomic.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
