# Task: cycle-insight-writer

Cycle: 0005
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0005

## Winners

- v001

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0005\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0005\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0005\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0005\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0005\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0005\hypotheses\H03.md

## Prior cycles digest

### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants targeted distinct axes inherited from Cycle 0001's open questions — constant-time LFU ordering, O(1) swap-based promotion, and cache-line alignment via padding — yet none cleared the Winners bar: the LFU bubble variant still regressed per-op-time on the get path, the swap variant hurt both hit rate and latency simultaneously, and the alignment variant recovered tail latency at the cost of a partial metadata-size give-back.

## Patterns

- **LFU hit-rate signal survives; the get-path overhead does not.** The bubble-sort LFU again demonstrated frequency-based eviction's advantage on scan-heavy traces, and eviction is now genuinely O(1) (tail removal). The remaining bottleneck is unconditional loop entry on every `get`, even when the accessed item's rank is already stable — a guard comparing the counter to its left neighbour before entering the loop is the natural next step.
- **Swap-to-back is a double loss at small associativity.** Replacing `remove+push` with `swap` failed on both dimensions: the bounded linear shift is not the latency bottleneck at small bucket sizes, and the ordering approximation degrades hit quality on the Zipfian workload. This axis should be abandoned.
- **Cache-line alignment is a confirmed, targeted fix for p99 spikes.** Padding the compact 24-byte struct to 32 bytes eliminated tail-latency straddling precisely as predicted, halving p99; the cost is 8 bytes of per-item footprint. At 32 bytes the struct still sits below the 40-byte baseline, making alignment padding the right structural foundation going forward even though it does not satisfy the metadata-size metric on its own.

## Open questions

- Can a neighbour-check guard (`if counter > left_neighbour.counter { bubble }`) on the LFU get path reduce unconditional overhead enough to clear per-op-time while preserving the hit-rate win?
- Does composing the 32-byte aligned struct with the guarded LFU bubble produce additive gains on both per-op-time and hit rate, or does the wider item footprint introduce new pressure that offsets the alignment benefit?
- Is there a way to reclaim the metadata-size dimension (currently 32 bytes vs. a 40-byte baseline) while holding the alignment fix — e.g., shrinking a different field rather than adding padding?

## Continuation

The next cycle should compose the two most promising pieces: the 32-byte aligned struct (confirmed p99 fix, already below baseline footprint) as the structural base, and the LFU bubble eviction with a stability guard that skips the bubble loop when the accessed item already outranks its left neighbour. This combination targets per-op-time and hit rate simultaneously without reintroducing the swap ordering regression. The swap-to-back promotion approach should be dropped entirely — it is not the latency bottleneck at these bucket sizes and actively harms hit quality.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants targeted the two open threads from Cycle 0002 — reducing get-path overhead via early-return guards and improving hit-rate quality via frequency-aware ordering — plus one structural experiment with two-zone partitioning. No variant reached the Winners bar: the position-0 guard traded latency for hit-rate, the two-zone partition regressed on both axes, and the single-swap LFU gained hit-rate without moving per-op-time.

## Patterns

- **Position-0 guard confirms the latency bottleneck but over-constrains promotion.** The early-return for already-hot items halved p99 latency as predicted, validating the write-elimination hypothesis. However, capping promotion to one swap per access prevents mid-bucket items from converging to the front quickly enough on a Zipfian workload, eroding the hit-rate the parent's full bubble had built. The useful insight is directional: the guard itself is sound, and the cap just needs to be wider than one.
- **Single-swap LFU with the aligned struct is a clean hit-rate gain at no latency cost.** Composing the 32-byte struct from Cycle 0002 with a frequency-ordered single swap lifted hit quality by steering eviction toward genuinely cold items without touching per-op-time — confirming that LFU ordering's advantage is eviction-candidate quality, not struct overhead, and that the aligned layout is a neutral-to-positive base.
- **Two-zone partitioning is a definitively closed avenue.** The hot/cold boundary shifted with every insertion, pushing boundary items back into the cold zone and degrading hit rate, while the hot-zone bypass saved no real work because baseline LRU already keeps hot items where removal is O(1). No reformulation of this design is worth pursuing at these bucket sizes.

## Open questions

- Can a bounded bubble loop (cap of 2–3 swaps) recover the hit-rate convergence that the single-swap variants sacrifice, while still short-circuiting the rare long-distance promotions that drive latency on a Zipfian workload?
- Does composing the position-0 guard with a bounded loop (rather than one swap) produce additive wins on both per-op-time and hit-rate, or does even a short loop reintroduce enough overhead to nullify the guard's p99 improvement?

## Continuation

The next cycle should compose three confirmed pieces: the 32-byte aligned struct as the structural base, the position-0 early-return guard for already-hot items, and a bounded promotion loop capped at 2–3 swaps rather than exactly one. This triplet directly addresses the failure mode identified in v001 (single-swap too conservative) while preserving v003's hit-rate quality and v001's tail-latency benefit. Two-zone partitioning should not be revisited. If per-op-time still does not improve after bounding the loop, the next hypothesis should examine whether the measurement floor itself masks the guard's contribution rather than adding further structural complexity.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
