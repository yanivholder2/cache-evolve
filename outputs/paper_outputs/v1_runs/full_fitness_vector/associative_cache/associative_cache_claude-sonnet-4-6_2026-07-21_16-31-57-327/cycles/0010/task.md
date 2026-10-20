# Task: cycle-insight-writer

Cycle: 0010
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0010

## Winners

- v002
- v001
- v003

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0010\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0010\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0010\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0010\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0010\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0010\hypotheses\H03.md

## Prior cycles digest

### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

All three variants attempted to decouple the second-chance eviction scan's hit-rate benefit from the 3-swap promotion cost by reducing swaps (to one, then to zero) or by tightening flag placement; none cleared the Winners bar, and v001 was selected as least-bad among a cohort that uniformly regressed.

## Patterns

- **Second-chance scan and 3-swap promotion are mechanically coupled, not independent.** Reducing to one swap or eliminating promotion entirely caused hot items to age into the scan window alongside cold ones, stripping the scan's ability to distinguish them. The eviction quality gain seen in Cycle 0006 is not a property of the flag alone — it requires promotion to maintain hot-item separation from the cold tail.
- **The 3-swap write cost is already below the 100 ns measurement floor.** Eliminating promotion swaps entirely bought no measurable per-op-time improvement, which means the latency regression attributed to "3-swap write pressure" in Cycle 0006's v003 originated elsewhere — the hypothesis that swaps were the bottleneck is now falsified.
- **Flag placement (first-touch vs. position-0 arrival) is neutral on this workload.** Once-accessed mid-range items were not a meaningful drain on eviction capacity under the 8-slot scan window; tightening the threshold produced no measurable hit-rate movement.
- **Compact 32-byte struct loss remains a recurring failure mode.** Variants that overlay only the eviction policy file without explicitly carrying forward the aligned item layout reintroduce per-item footprint bloat and put-path latency — this has now surfaced in multiple cycles and must be treated as a required invariant, not an incidental fix.

## Open questions

- If 3-swap write cost is below measurement resolution, what actually caused the put p99 regression in Cycle 0006's v003 — could it have been an artifact of a different structural change in that variant, such as a scan-window size or a metadata layout detail?
- Is there a write-free way to maintain the hot-item / cold-item separation that the 3-swap loop currently provides, or is positional movement strictly necessary for the second-chance scan to function?

## Continuation

The decoupling hypothesis has been tested exhaustively and rejected — the next cycle should abandon further attempts to separate the second-chance scan from the 3-swap loop and instead treat their combination as a confirmed unit. The productive path is to return to the Cycle 0006 v003 configuration (3-swap + write-once flag + 8-slot second-chance scan + compact 32-byte struct) and diagnose the latency regression source directly, since the swap-elimination experiments have ruled out write pressure as the cause. Compact struct inheritance must be enforced in every variant going forward.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants probed the second-chance scan's internal parameters — window depth and scan direction — rather than its interaction with promotion. The two winners confirmed that the 8-slot window is already near-optimal in depth, and that reversing iteration order to evict the oldest cold candidate first is a free hit-rate gain.

## Patterns

- **Scan direction is the highest-value tunable found in the second-chance scan.** Switching from newest-cold-first (LIFO-among-cold, tail position) to oldest-cold-first (LRU-among-cold, inner edge of window) improved hit rate at zero latency cost. The parent's LIFO ordering was silently discarding items too young to have received a second request; LRU-among-cold preserves them long enough to convert.
- **8-slot scan depth is already the sweet spot; neither direction of change helps.** Halving to 4 positions degraded p99 latency without any hit-rate movement — on the rare evictions where cold items don't occupy the bottom 4 positions, the narrower window falls back to a worse candidate and spikes tail latency. Expanding to full bucket depth added unbounded comparison work with zero hit-rate return, confirming that never-accessed items beyond the 8-slot tail are essentially absent under the 3-swap promotion regime.
- **Scan cost is positional, not directional.** The iteration-order flip in v003 carries no measurable overhead because scan length is unchanged; all the per-call work budget is in the traversal count, not the comparison order.

## Open questions

- Is the LRU-among-cold ordering optimal, or would a frequency-weighted tie-break within the cold window (e.g., evict the item with the fewest accesses among the cold candidates) add further hit-rate lift on this workload?
- With scan direction confirmed fixed and scan depth confirmed fixed, is there remaining headroom in the second-chance flag's definition — e.g., expiring or resetting the "ever-accessed" bit after N eviction passes to prevent stale warm markers from permanently shielding aging items?

## Continuation

The next cycle should treat the 3-swap loop + 8-slot second-chance scan + LRU-among-cold eviction order as the new confirmed base, replacing the prior LIFO ordering throughout. The remaining structural unknown from Cycle 0007 — what actually caused the put p99 regression in Cycle 0006's v003 — has not been resolved and should be targeted directly: compare the new base against the original 8-slot LIFO configuration on put-heavy access patterns to isolate whether the regression was a metadata-layout artifact, a scan-fallback artifact, or something else. Flag-lifetime and frequency-within-cold experiments are the natural next frontier if the put regression diagnosis comes back clean.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

All three variants attempted to refine the second-chance scan's eviction quality — by adjusting the warm-all fallback index, introducing a two-generation access counter with priority passes, or expiring warm flags on deep-cold displacement — and all three regressed. v003 was selected as least-bad among a uniformly regressing cohort.

## Patterns

- **The base's LRU-among-cold fallback index (`len - 1`) was already correct; the H01 hypothesis had the layout direction inverted.** In this cache's positional model, higher index means less recently used, so `len - 1` is the genuine LRU tail. Changing the fallback to `len - scan_depth` selected the *most recently* inserted item in the cold window — anti-LRU — explaining the hit-rate drop without any algorithmic flaw in the parent.
- **Single-access items carry real reuse probability on a Zipfian workload.** Elevating them as eviction targets (generation-1 priority pass in v002) drained items that would have converted to confirmed-warm items on their next request. The second-chance flag's binary (ever/never) distinction already captures the meaningful split; finer-grained generation counts add classification cost without eviction benefit under this distribution.
- **"Deep-cold position" is not a reliable proxy for "stale warm marker."** On Zipfian access, legitimately warm items accumulate near the cold tail simply through low-but-nonzero access frequency. Expiring flags based on position alone (v003's H03) conflates low frequency with staleness, evicts genuinely reusable items, and introduces an extra conditional store that raises tail latency even on a rarely-firing path.
- **Compact struct omission is still recurring.** v002 silently dropped `item.rs` from its overlay and reverted to the 40-byte baseline layout — the third cycle in a row where this invariant has been violated. Variants that touch only `eviction_policy.rs` must explicitly carry the `item.rs` overlay forward.

## Open questions

- The put p99 regression from Cycle 0006's v003 has never been directly diagnosed; the swap-elimination experiments in Cycle 0007 ruled out write pressure, but the actual cause — metadata layout artifact, scan-fallback behavior, or otherwise — remains open.
- Has the 3-swap + 8-slot + LRU-among-cold + write-once-flag base converged near a local optimum for this workload, or is there a structurally different axis (e.g., adaptive scan depth, insertion-point policy) that hasn't been explored?

## Continuation

The three axes probed in this cycle — fallback-index tuning, generation-counter granularity, and flag expiry on displacement — have all been tested and rejected; the next cycle should not revisit them. The most tractable unresolved thread is the Cycle 0006 put p99 regression: the next hypotheses-writer should design a direct A/B comparison between the current confirmed base and the original 8-slot LIFO configuration on put-heavy access patterns, isolating whether the regression was tied to the LIFO ordering, the compact struct transition, or a scan-fallback edge case. If that diagnosis clears, insertion-point experiments — placing new items somewhere other than the conventional tail position — are a structurally distinct axis worth opening.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
