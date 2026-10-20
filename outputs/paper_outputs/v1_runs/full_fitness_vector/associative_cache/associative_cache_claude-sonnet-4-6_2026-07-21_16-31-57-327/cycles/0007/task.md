# Task: cycle-insight-writer

Cycle: 0007
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0007

## Winners

- v001

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0007\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0007\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0007\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0007\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0007\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0007\hypotheses\H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants composed pieces recommended by Cycle 0003 — a bounded promotion loop, an unconditional recency advance, and a two-tail eviction selector — with each producing a hit-rate lift but none clearing the Winners bar, primarily because per-op-time remains pinned at the benchmark's timer-resolution floor regardless of approach.

## Patterns

- **Three-swap cap resolves the single-swap bottleneck cleanly.** Extending the promotion budget to three swaps lets mid-bucket items reach the head zone without per-op-time cost, because the early-break on a failed swap means the common Zipfian hot-key case exits after one iteration. Cycle 0003's single-swap cap was the precise constraint; three swaps is sufficient to close the convergence gap.
- **Unconditional recency promotion beats frequency-gated promotion on this workload.** Removing the `lfu_counter` neighbour check and always advancing one step outperformed soft-LFU ordering: the frequency gate was stranding recently-reused items below cold neighbours, and always-advance keeps the tail genuinely cold for the eviction path.
- **The 32-byte aligned struct is a non-negotiable base.** Dropping the compact layout in v003 immediately inflated per-item footprint and likely reintroduced cache-line straddling, erasing the two-tail eviction benefit. Every future variant must carry the alignment fix forward explicitly.
- **The per-op-time measurement floor is now a visible ceiling.** Both v001 and v002 show no latency regression, but neither shows improvement; v002 explicitly attributes this to timer resolution. The benchmark's floor is masking whether any latency gains are real, which means further per-op-time optimisation hypotheses cannot be evaluated until timer resolution improves.

## Open questions

- Do the 3-swap loop (v001) and the always-advance rule (v002) address the same mid-bucket convergence bottleneck from different angles, or are they additive when composed?
- Is the two-tail eviction comparison (v003) worth re-testing in combination with the compact struct, and does it compose cleanly with either promotion strategy?
- Is the 100ns timer floor a benchmark configuration issue, or does meaningful latency differentiation require a workload with higher operation count / longer trace to amortise measurement noise?

## Continuation

The next cycle should compose the 32-byte aligned struct with the 3-swap bounded loop (v001) and test whether replacing the frequency gate with the unconditional always-advance rule (v002) produces additive hit-rate gains or converges to the same outcome. The two-tail eviction selector is worth a clean re-test paired with the compact layout. If per-op-time results remain flat after these compositions, the cycle after that should explicitly investigate whether the measurement floor is the blocker rather than introducing new structural changes — resolving this ambiguity is now more urgent than adding further promotion complexity.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
