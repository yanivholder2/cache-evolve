# Task: cycle-insight-writer

Cycle: 0009
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0009

## Winners

- v003

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0009\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0009\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0009\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0009\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0009\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0009\hypotheses\H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cohort probed three independent axes: promotion-budget saturation, bucket-count scaling, and frequency-informed eviction via a write-once binary flag. Both winners stayed on the 3-swap positional base — one confirming its ceiling, the other pairing it with a second-chance tail scan that measurably improved hit rate.

## Patterns

- **3-swap promotion budget is confirmed optimal; the ceiling is now mapped.** Extending to five swaps produced a slight hit-rate regression, meaning items beyond three hops from the head are accessed rarely enough on this Zipfian workload that forcing them forward displaces items that cycle through the top positions. The useful range of promotion-budget exploration is exhausted.
- **Second-chance eviction with a write-once flag is the most promising hit-rate lever found so far.** A binary "ever-accessed" marker — written once at first touch, never again on the hot get path — lets the eviction scan skip past genuinely warm items and eject cold newcomers instead. This directly answers Cycle 0005's call for frequency information without per-get writes.
- **The latency regression in v003 traces to the 3-swap loop, not the flag.** The second-chance scan itself is inexpensive; it is the unconditional position-update swaps on every hit that push per-get write cost above the baseline. The hit-rate gain and the latency cost are therefore separable — they come from different mechanisms.
- **Halving bucket count is a definitively closed direction.** Fewer buckets raise collision density faster than added depth can absorb, and the 3-swap budget cannot rescue items stranded beyond position three in doubled-width buckets. The hit-rate regression replicates Cycle 0001's findings and removes any remaining ambiguity.

## Open questions

- Can the second-chance eviction scan be paired with a cheaper promotion strategy (fewer swaps, or none at all) to preserve the hit-rate lift while eliminating the per-get write regression?
- Does a zero-swap get path — pure positional read with no item movement — combined with the second-chance eviction scan still outperform strict LRU, or does recency ordering degrade too quickly without any promotion?

## Continuation

The next cycle should decouple the two mechanisms in v003: keep the write-once binary flag and second-chance tail scan, but reduce or eliminate the per-get promotion swaps. Specifically, try pairing the second-chance scan with (a) a single swap only — recovering the position-0 guard pattern from Cycle 0003 — and (b) no promotion at all, relying entirely on eviction quality rather than positional convergence for hit-rate. If either pairing holds hit rate near v003's level while restoring latency to the v001 baseline, it becomes the new confirmed base. Bucket count and promotion budget beyond three swaps should not be revisited.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
