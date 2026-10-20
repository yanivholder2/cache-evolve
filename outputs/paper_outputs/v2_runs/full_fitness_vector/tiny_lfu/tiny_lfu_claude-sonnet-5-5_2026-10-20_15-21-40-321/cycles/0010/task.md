# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0010

## Winners

- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Three non-sketch changes were tried on different parents: pre-sizing the map and slab with in-place slot reuse (v001), inlining and branch hints on the `get` fast path (v002), and a segmented probation/protected main LRU on the baseline (v003). All three were kept as Winners. Only v003 showed a plausible real change: a smaller Node and faster get and put, with a slight hit-rate loss. v001 and v002 were flat or worse.

## Patterns

- Allocation and growth are not a bottleneck. Pre-sizing and slot reuse changed nothing measurable, because the cache fills early and steady-state growth cost is negligible.
- Forcing inlining and branch hints on helpers shared by `get` and `put` did not help and may have slowed both. Call overhead, duplicate hashing and branch layout are not where `get` spends its time. The broad slowdown may also be noise.
- Segmenting the main LRU did not raise hit-rate. The sketch admission filter already supplies most of the scan resistance this trace needs, so main-region scan resistance is not the hit-rate limiter.
- v003's speed and size gain came from the compact Node, with segment membership packed into a spare bit, not from the segmentation itself. This again points to memory footprint and locality as the lever that pays.
- Hit-rate was flat or slightly down across the cohort, so the plateau on workload-a holds against a structural mechanism too.
- Two insights were written without a source diff, so some attributions are inferred. Small latency deltas remain unreliable without repeat runs.

## Open questions

- Is v003's speed gain driven by the compact Node alone? If so, can it be reproduced on the stronger incumbents without the segmentation and its hit-rate cost?
- Where does time actually go in `get` and `put`? There is still no profile.
- Is any hit-rate lever left on workload-a in window sizing or the admission decision itself?

## Continuation

Drop pre-sizing, `#[inline(always)]` and branch-hint tweaks, and drop segmenting the main LRU as a hit-rate idea. Pursue footprint and locality: carry the compact 24-byte Node layout onto the best incumbent without the segmentation, and test whether the speed gain survives repeat runs. For hit-rate, look at window sizing or the admission decision, and have each hypothesis name which cost it expects to move.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

Three changes were tried: halving the sketch table (v001), shrinking the window to a smaller share of capacity (v002), and a compact Node with u32 slab-index links and packed flags, without segmentation (v003). All three were kept as Winners, but none improved speed or hit-rate. Only v001 and v003 gave a real footprint reduction, and each paid for it with slower get and put.

## Patterns

- Footprint reductions did not turn into speed. Both size cuts, one in the sketch and one in the Node, left latency worse. The working set seems small enough that locality gains are absent. The index-conversion and flag-unpacking overhead is paid on every operation.
- v003 undercuts the cycle-0007 lesson. The compact Node alone gave no speed gain, so cycle 0007's speedup came from something else, or was noise. The "footprint and locality pay" thread is now weak.
- Window size is not a hit-rate lever. A smaller window lost a little hit-rate and slowed put, since more inserts go through admission and eviction. This closes the window-sizing question from cycle 0007.
- Hit-rate has stayed flat or slightly down again across the cohort. Sketch, window, segmentation and Node layout have all failed to move it on workload-a.
- v002 showed unexplained growth in Node and struct size that the change should not cause, so its overlay may carry an unintended edit. Its attributions are inferred.
- Latency regressions are consistent in direction across v001 and v003, which makes them somewhat more believable than earlier noise-level deltas. Winner status here reflects the selection rule, not real gains.

## Open questions

- Where does time go in get and put? There is still no profile, and every blind hot-path guess so far has been flat or worse.
- Is the admission decision itself (compare incoming frequency against victim frequency) the only remaining hit-rate lever, or is workload-a at its ceiling for this design?
- Are the incumbents at the floor for this architecture, so that further gains need a different data structure?

## Continuation

Drop sketch size, window size, and Node compaction as standalone levers. They are all explored. Do not build on v002 without auditing its diff. Next cycle should either obtain a profile-driven target (map lookup, relinking or sketch estimate) before proposing speed changes, or try a structurally different admission or data-structure idea, such as an adaptive window or a cheaper hash map. Each hypothesis should name the specific cost it expects to move and be checked with repeat runs.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Three hypotheses were planned (adaptive window, relink-skip plus victim-estimate cache, flat open-addressing table with a slab and single hash). v001 was rejected: its implementer hit a backend "prompt too long" failure and produced nothing, so the adaptive window is still untested. Winners were v002 and v003. v003 gave a real, large latency gain. v002 was a regression that passed only on the selection rule.

## Patterns

- The only substantive win is a storage-layer change. Replacing the pointer-based map with a flat linear-probing table over a slab, hashing once per operation and reusing that hash for the sketch rows, cut get and put latency at median and tail. Hit-rate, Node size and policy stayed unchanged.
- This is the first blind hot-path guess in several cycles that paid off, and it contradicts the cycle-0008 worry that incumbents sit at the floor. The map and hashing path, not sketch, list or Node layout, was the dominant per-op cost on the baseline.
- v002 confirms that relinking and repeated victim estimates are not the cost. The epoch is bumped on nearly every access, so the victim cache almost never hits, and the bookkeeping slowed both paths and grew the cache struct. Hit-rate was unaffected.
- Hit-rate remains flat across every cohort, including a pure storage change. Metadata size is dominated by the sketch, so storage work cannot move it.
- v003's parent was the baseline, not the strongest hit-rate incumbent, so its gain is not yet shown to stack on the later lineages.

## Open questions

- Does the flat-table and single-hash gain carry onto the cycle-0004 v003 or cycle-0007 lineages? Cycle 0005 showed that a baseline-only speedup can fail to transfer.
- Is the v003 gain mostly from the flat table or from the single hash feeding the sketch? Earlier single-hash derivation failed to stack on strong parents.
- Does the adaptive window do anything for hit-rate on workload-a? It was never evaluated.

## Continuation

Port v003's slab and flat-table storage onto the best hit-rate incumbent, and separate the table's effect from the single-hash reuse, with repeat runs. Re-propose the adaptive hill-climbing window (H01) once, since it was never tested. Drop the victim-estimate cache and near-head relink skipping, and keep hypotheses aimed at the map and hashing path rather than the sketch or list.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
