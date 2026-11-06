# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants changed ClassicLRU inside the bucket, each through a different lever: where new items are inserted (v001, LIP-style quarter-depth), how hits are recorded (v002, CLOCK reference bit), and which item is picked as the victim (v003, frequency-aware tail window). The Winners, v001 and v002, both gained hit rate by making new one-touch items the first to be evicted. v003's hit rate came out identical to the baseline, which most likely means its change never ran in the hit-rate harness.

## Patterns

- Scan resistance is the hit-rate lever that worked. Both Winners make new items "earn" their place before they are protected from eviction, either by position or by a cleared bit. The gain is biggest with the wide (1024-item) buckets, where each bucket acts most like a full LRU.
- Every variant made put latency worse, especially the tail. Any extra work on the miss/eviction path (mid-bucket insert, a per-item sweep, a window scan plus aging) cost more than the single bulk memmove it replaced or added to. The assumption that "a memmove is a memmove" did not hold.
- Get p50 barely moved in any variant, even when the hit path became a single store (v002). So most of the get cost is the linear key lookup across the wide bucket, not the reordering.
- The `name() == "ClassicLRU"` string compare on every op appeared in two variants and adds overhead to the put path that has nothing to do with the policy.
- None of the variants changed metadata-size. Reusing `lfu_counter` and changing only the insert position are both free on that axis.

## Open questions

- Does the hit-rate harness actually run the overlaid bucket `evict_item` / ClassicLRU code paths? v003's bit-identical result needs checking before frequency-aware victim selection is judged.
- Does v001's quarter-depth insert combine with v002's cleared reference bit, or do they overlap and give the same scan protection twice?
- Should the insertion depth scale with bucket width, given the weak gain at the smaller size?

## Continuation

Build on the scan-resistant insertion idea (v001/v002). Tune or scale the insertion depth, and decide the policy check once at construction instead of comparing strings on every op. Target speed directly at the linear `get_position` scan (for example a tag array or per-bucket key index) rather than at reordering cost, and avoid unbounded per-item eviction sweeps. Before retrying v003's frequency-window idea, first confirm that the harness runs the overlaid eviction path.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Two variants refined last cycle's scan-resistant policies: bimodal insertion on top of LIP (v001) and a 2-bit GCLOCK with a bounded sweep (v002). The third variant (v003) went after the lookup scan with a dense key mirror. v001 is the only Winner, and only just: its hit-rate summary edged up because of one cell (hm_0 at the 128-wide size), while it lost slightly on web_0 and put latency got worse.

## Patterns

- Hit-rate gains from changing the replacement rule are now small, and each one trades between workloads. BIP's blind MRU admission gives back scan resistance on web_0. Extra GCLOCK levels add only a little hit rate, though they did so at every point. The easy scan-resistance gains were taken in cycle 0001.
- Put latency is still set by the mechanism, not by policy dispatch. Removing the per-op string compare (v001) had no visible effect, because the mid-bucket memmove dominates. Capping the sweep (v002) and scanning dense keys (v003) both helped the put tail. Each also made the typical put slower, through more full-window sweeps or writes to two allocations.
- The get p50 summary got worse in both speed-oriented variants. In v003 the cause was clearly locality: a separate key Vec adds a second memory stream to every hit, and on short scans that costs more than density saves. Splitting hot data across allocations is the wrong shape of fix.
- Tail and median now move in opposite directions. Changes that bound worst-case work (sweep cap, dense absence scan) improve p99, and every one so far has added per-op cost at p50.
- Metadata-size stayed flat again. v003 showed that this dimension cannot see memory held in side arrays inside the bucket.

## Open questions

- Is v002's small get p50 regression real or noise? Its get-path change was tiny.
- Can a period that scales with bucket size, or a reuse-aware MRU admission signal, keep BIP's small-bucket gain without losing web_0?
- Would a per-slot fingerprint or tag array stored in the same cache line, or a dense reference-bit array for the sweep, cut get p50 where a separate key Vec did not?

## Continuation

Keep v002's 16-slot sweep cap and clear-on-insert as the base mechanism for the put tail. Treat further replacement-rule tweaks as low yield unless they target a specific workload (for example, BIP admission that scales with bucket size). Spend most of the next cycle on get p50 with layouts that keep the key and the hit-time data together (inline tags or fingerprints, SIMD-friendly blocks), and drop the separate-Vec key mirror. For LIP-style insertion, look for a cheaper insert-at-depth rather than trimming dispatch overhead.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

Two variants tried to shorten the get scan by changing scan order or key placement: v001 reversed the scan from the MRU end under LIP/BIP, and v003 used swap-halfway self-organization under CLOCK. The third (v002) replaced BIP's fixed-period MRU admission with a 16-entry ghost ring. v001 and v002 are the Winners, but both wins are marginal. v001 changed nothing measurable beyond noise. v002 traded the hm_0 small-bucket gain for web_0 large-bucket scan resistance and paid for it in put latency.

## Patterns

- Changing scan order or key placement does not move get p50. Reversing the scan gave about a noise-level gain. Self-organizing swaps made it clearly worse, because they turned CLOCK's one-byte hit store into writes across two cache lines. Across three cycles, the lookup cost tracks how many full-key compares are done, not where the hit sits. Hits under quarter-depth insertion are not concentrated at either end.
- Hit-path writes must stay as cheap as a bit store. Any reordering on hits (v003) gives back CLOCK's latency advantage and also slightly disturbs the hand's eviction order.
- A reuse signal only works if its window matches the workload's reuse distance. A fixed 16-key ghost catches nothing on hm_0's slow-shifting set, so on small buckets it behaves like plain LIP. It does correctly keep scan keys out of MRU on web_0. Neither BIP's blind period nor the short ghost wins on both workloads.
- Every extra structure on the put-miss path again raised put p50 and p99. The `Vec::insert` memmove at quarter depth is still the main cost, with the new work added on top.
- The metadata-size metric is blind to memory held inside each bucket. This is now confirmed twice (key mirror, ghost ring), so do not treat flat metadata as evidence that a change is free.

## Open questions

- Would a ghost window sized to `max_size` (or a hashed bit-set reuse filter) catch hm_0 reuse on 128-wide buckets without losing web_0?
- Does a compact inline tag/fingerprint pre-filter actually cut get p50? All three cycles' insights point to it, but it has not been tested yet.
- Can quarter-depth insertion avoid the memmove, for example with a segmented or CLOCK-like approximation of LIP, so that put latency stops growing with every policy tweak?

## Continuation

Stop changing scan order and hit-time reordering: both are exhausted. Make the next speed hypothesis a compact fingerprint/tag pre-filter that reduces full-key compares while keeping the hit-path write a single store. For hit rate, pursue a reuse filter that scales with bucket size (a sized ghost or a hashed bit set) as a replacement for both BIP's period and the fixed 16-slot ring. Separately, target the cost of the insert-at-depth memmove. Treat flat metadata-size as uninformative whenever a change adds per-bucket state.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

Three variants worked on three separate axes. v001 merged the full-bucket evict and quarter-depth insert into one partial rotation, aiming at put latency. v002 replaced BIP's period with a hashed ghost bit set sized to the bucket, aiming at hit rate. v003 narrowed `Item`'s bookkeeping fields, aiming at metadata-size. v003 is the Winner because it is the first variant in four cycles to move metadata-size, with hit rate exactly unchanged.

## Patterns

- Per-op-time could not be trusted this cycle. All three variants got slower on get, including v001, whose get path is byte-identical to its parent. That points to a shift in the benchmark environment rather than to any one change. As a result, no speed hypothesis was actually tested: v001's predicted memmove saving and v003's predicted cache-line gain are both still open.
- Rewrites that keep the layout are reliable on hit rate. v001 (fused rotation) and v003 (narrow fields) both matched every hit-rate cell exactly, so the parent's policy can now be treated as a stable base to build on.
- New hot-path costs keep coming in with changes meant to be cheap. Examples: a dynamic policy call on every miss (v001), SipHash twice per miss plus periodic full clears (v002), and a global atomic clock on every put (v003). Each one plausibly cancels the gain it sits next to. Any work added to the put path must be per-bucket and non-atomic, or decided once at construction.
- A reuse filter has to be sparse to be useful. At about one bit per tracked eviction, v002's filter fills up and acts like a high-rate random MRU promoter. That hurt hm_0 at both widths and web_0 on wide buckets, worse than BIP's blind 1/32. Its all-at-once reset also starves shifting working sets. This is the second reuse-signal design to fail on the two-workload tradeoff.
- Metadata-size can move after all: it measures `Item` layout, not per-bucket state. Shrinking the item is the one lever that reaches this dimension.

## Open questions

- With the benchmark rerun against its parent in the same session, does the fused rotation (without the dynamic victim check, and using `ptr::copy`) actually cut put latency?
- With the 24-byte item kept but the shared atomic clock removed (a per-bucket tick, or no write at all), does the denser layout speed up the scan?
- Is any reuse filter worth another attempt? It would need several bits per key, gradual aging and a cheap integer hash, and it is still not clear it can beat BIP on both workloads.

## Continuation

Build on v003's 24-byte `Item`, but replace the global atomic tick with a per-bucket counter, or skip the write when the policy never reads it. Then layer v001's fused rotation on top, cleaned up (victim-is-index-0 cached at construction, `ptr::copy` shift). Judge both against a same-session parent baseline, since this cycle's per-op-time numbers are unreliable. Deprioritize hashed ghost filters. The inline fingerprint/tag pre-filter for get p50, recommended for two cycles now, is still untested and should be one of the next hypotheses.


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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants branched from cycle 0007/v003, the clock-free fused-rotation base on byte tags. v001 hand-wrote the hit-path MRU shift with `ptr::copy`. v002 ported width-keyed BIP to this base at 1/64 on wide buckets. v003 replaced the byte tag scan with a SWAR scan. v002 and v003 are Winners. v002 gave a small net hit-rate gain. v003 is a Winner only because its invariants held: get and put both got clearly slower. Neither speed hypothesis produced a speed win.

## Patterns

- Both speed rewrites backfired at p50 on get and put. These are large, same-direction moves, not tail noise. In both cases the rewrite swapped a simple loop the compiler handles well for "clever" code. The variable-length `ptr::copy` probably became an out-of-line `memmove`, and SWAR added fixed per-chunk work plus a branchy mask loop. The put win from cycle 0006 does not generalise. Hand-written moves pay off only when they remove whole memmoves, not when they replace a short inline move.
- Counting compares stopped working as the cost model. The tag pre-filter won in cycle 0005 by cutting full-key compares. Making the byte scan itself wider did not help, which suggests typical scans are short or already vectorised. The remaining miss-path cost is probably not in the tag scan.
- Two caveats on the evidence. The v001 and v003 readings were written without access to the diffs or the generated assembly, so the proposed mechanisms (memmove call, de-inlining) are inferences. And get slowed in the hit-rate-only v002 too, so part of every per-op move this cycle may still be drift.
- The wide-bucket BIP probe is closed. Going from 1/32 to 1/64 to 1/128 moves the web_0 gain and the hm_0 loss roughly in proportion, so there is no sweet spot. The hypothesis's own stop rule fires. Width-keyed masking is still worth keeping as a zero-metadata way to split policy by bucket size: narrow cells stayed bit-identical again.
- Hit-rate and metadata-size invariants held exactly for all three variants. Mechanical rewrites remain semantically safe, so the risk is now entirely in per-op time.

## Open questions

- Does v001's slowdown come from a `memmove` call or de-inlining? Would a fixed-width or const-generic copy, or a fused swap loop, fix it? Is the parent's hit path already near-optimal?
- How far does a typical tag scan actually go before it matches or ends? If the answer is "short", where does the rest of the miss-path time go: the key compare, the rotation, or the bucket indexing and hashing?
- How much of this cycle's latency movement survives repeated same-session runs against the parent?

## Continuation

Adopt v002 (width-keyed BIP, 1/64 wide) as the new base and freeze the policy. Stop tuning the BIP period. Drop SWAR and other hand-widened tag scans. Drop variable-length `ptr::copy` on the hit path, unless an assembly check shows a fixed-width copy would stay inline. The next speed hypotheses should begin by locating the real cost: profile scan length, check codegen, and look at hash/indexing cost. Judge them only with repeated same-session runs against the parent.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Each variant tried to remove a whole unit of work. v001 (from 0007/v003) cached a per-bucket "confirmed absent" result so a put could skip the tag scan. v002 (from 0008/v002) skipped the MRU move on hits in the top eighth of the bucket (lazy promotion). v003 (from 0008/v002) replaced the cache-level SipHash bucket index with one multiplicative hash that supplies both the bucket bits and the tag. All three are Winners. v002 and v003 are the first clear speed wins since cycle 0006: get and put both got faster at p50 and p99, and hit rate stayed well inside budget. v001 counts as a Winner only because its invariants held. Put got clearly slower and get p99 rose.

## Patterns

- Cycle 0008's question about where the miss-path time goes now has an answer: a large share sat above the bucket, in a redundant SipHash per op. Cycles 5–8 tuned in-bucket loops and never reached it. v003's gain is the most credible of the series because it shows up on both ops and both percentiles, and both ops shared the hash it removed.
- "Remove whole work" only wins when the removed work costs more than the state needed to skip it. v002 needed no state and won. v001 added fields, compares and stores to skip a cheap scan that was already in L1, and lost. The tag scan is now cheap enough that it should not be targeted again.
- Lazy promotion worked where earlier hit-path rewrites failed (`rotate_left`, `ptr::copy`). It removes the move for near-MRU hits instead of re-implementing the move. Reordering inside the top window is far from the eviction end, so hit rate barely moved.
- Writes on read paths are costly. v001 made `get_position` store into the bucket, which dirtied the cache line on every miss, and it grew the stride from 80 to 88 bytes. The get-tail rise is consistent with that. Keep lookups read-only and keep the stride pinned.
- Re-mapping buckets with a multiplicative hash on bits 24–55 did not cluster strided block-address keys. Hit rate moved slightly upward. A cheap hash is safe for these traces.
- v002's put gain is partly drift: the put-miss path is unchanged and only put-update benefits. Same-session repeat runs are still required.

## Open questions

- Do v002 and v003 stack when merged? Their effects should be independent (the hash cost above the bucket, the hit-path memmoves inside it).
- Does widening the lazy-promotion window to `n/4` skip more moves without reaching the quarter-depth insertion region, where eviction order starts to change?
- Now that both hashing and hit-path moves are reduced, what is the largest cost left: the full-key compare, the fused rotation on put-miss, or bucket indexing (`%` vs mask)?
- Does stride by itself cost anything? This needs a pad-only control (parent padded to 88 bytes with no behaviour change), and v001 could not isolate it.

## Continuation

Merge v003's single multiplicative hash with v002's lazy promotion into the new base, keep stride at 80, and verify it against both parents with repeated same-session runs. Next, probe a lazy-promotion window of `n/4` with a stop rule at the insertion region, then profile what remains above and inside the bucket. Drop miss memos, any other state added to skip cheap scans, and any writes on the get path. Keep the policy frozen except for the promotion window.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
