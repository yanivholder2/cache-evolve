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
