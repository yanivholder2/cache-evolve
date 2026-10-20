# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0009

## Winners

- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

Three sketch-only changes were tried on existing incumbents: a single-hash shift-derived index (v001, on the cycle-0004 v003 parent), a saturation early-exit alone (v002, on the cycle-0004 v001 parent), and conservative-update increments (v003, on the cycle-0005 v002 parent). All three were kept as Winners, but none gave a real improvement. Hit-rate and metadata size stayed flat, and per-op time did not improve.

## Patterns

- The cycle-0005 single-hash speedup did not carry over to the stronger lineage. On this parent it made `get` slightly slower and may have cost a hair of hit-rate, since slicing indices from one hash correlates the rows. The earlier gain was therefore specific to the baseline lineage.
- The early-exit alone did nothing measurable. Counters rarely sit at the cap, so the early-exit rarely fires. Any speedup in the cycle-0005 variant that combined both changes probably came from the hash derivation, and that derivation has now failed to stack.
- Conservative update left hit-rate unchanged. Collision inflation is not what limits hit-rate on workload-a, which is the third sketch-accuracy result (doorkeeper, width, conservative update) to say the same thing.
- Per-op shifts across the cohort were small and in both directions. Most are best read as noise, as in earlier cycles.
- Sketch cost no longer looks like the main per-op term on the current incumbents. Sketch tweaks of every kind (accuracy, width, increment cost) are now exhausted.
- Two insights were written without a source diff, so some attributions are inferred.

## Open questions

- Where does `get` actually spend its time on the incumbent: map lookup, list relinking, or sketch estimate? No profile exists.
- Is any remaining gain in the non-sketch hot path (relinking, allocation, bounds checks), or are the incumbents at the floor for this design?
- Is there any hit-rate lever left on workload-a, given that sketch accuracy, victim choice, tie rules, aging, windows and doorkeeping have all failed?

## Continuation

Drop all sketch-internal changes (hash derivation, early-exit, conservative update, width, doorkeeper). Aim the next cycle at the non-sketch hot path of the cycle-0004 v003 incumbent (map access, list relinking, bounds checks), or at a structurally different admission mechanism such as a window or segmented layout, and have each hypothesis name which cost it expects to move. Repeat latency runs before crediting any small delta.


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


## Rejected variants

- v001 — implement
  agent 'variant-implementer' returned status=ok but recorded 0 input AND 0 output tokens (usage=input_tokens=0, output_tokens=0, cache_read_tokens=0, cache_write_tokens=0, cost_usd=0) — almost always a silent backend failure (session limit, quota, dropped connection). tail of /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0009/variants/v001/attempts/02/variant-implementer.log.md:
  # Agent log: variant-implementer
  
  - adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
  - workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0009/variants/v001/attempts/02
  - allow_edits: True
  - started: 2026-10-20T15:55:59+00:00
  
  ## timeline
  
  ### 15:56:02 chat
  
  Prompt is too long
  

## Write

Exactly one file: `cycle-insight.md` in the output dir.
