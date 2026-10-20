# Task: cycle-insight-writer

Cycle: 0008
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0008

## Winners

- v003
- v001

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0008\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0008\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0008\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0008\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0008\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0008\hypotheses\H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants probed promotion strategy and frequency-signal placement: one extended the unconditional swap budget, one added a frequency overlay at the eviction boundary, and one gated promotion distance on an access counter. Only the purely positional, write-free approach (v001) landed in Winners.

## Patterns

- **Unconditional multi-swap promotion wins cleanly at no latency cost.** The 3-swap loop's early-break keeps the hot-path at one iteration for already-head items, so Zipfian workloads see the full convergence benefit with no measurable overhead. Promotion budget expansion from one to three swaps was the right lever; the convergence lag at single-step is real and now resolved.
- **Frequency signals on the get path carry consistent tail-latency risk.** Both variants that wrote `lfu_counter` per access surfaced p99 pressure — v002 from counter writes hurting eviction ordering, v003 from the combined weight of counter writes plus up to three additional swaps. The write itself, not the conditional logic, appears to be the cache-line liability on the get path.
- **Frequency overlays at the eviction boundary degrade hit rate on this workload.** Two-tail frequency comparison (v002) overrode clean positional recency, ejecting recently-promoted items before genuine LRU victims. Strict tail eviction remains the correct policy; no frequency signal applied at eviction time has improved upon it across two cycles of testing.

## Open questions

- Does promotion budget beyond three swaps (e.g., four or five) continue to gain hit rate, or have diminishing returns already set in at three given the Zipfian concentration of hot keys near position 0?
- v003's p99 doubling is the first latency signal to clearly clear the measurement floor — does it indicate a general threshold on per-get write volume, and can that threshold be quantified to bound future frequency-integration experiments?

## Continuation

The next cycle should build on v001's 3-swap unconditional loop as a confirmed base and investigate whether frequency information can be incorporated without any write on the hot get path — for example, sampled or epoch-based counters that update only on eviction or put, not on every get. Two-tail eviction and any per-get counter write should be treated as closed unless a fundamentally different encoding is proposed. If hit rate appears to have plateaued after further promotion-budget trials, the focus should shift to understanding the measurement floor more precisely, as v003's p99 signal suggests latency differentiation is now observable under write-heavy get paths.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
