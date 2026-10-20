# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0008

## Winners

- v002
- v001
- v003

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Three footprint and speed changes were tried: reverting the 4x wider sketch on the cycle-0004 parent (v001), dropping a cached field from Node on the other cycle-0004 parent (v002), and a single-hash shift-derived sketch increment with a saturation early-exit, rooted at baseline (v003). All three were kept as Winners, but only v003 gave a plausible real gain (cheaper put and get medians, hit-rate level). v001 and v002 gave no real improvement.

## Patterns

- Sketch width is a weak lever in both directions. Narrowing saves hidden heap and costs a hair of hit-rate. The predicted locality speedup did not appear, and latency got slightly worse.
- v001's slowdown, and v002's get "gain" from an untouched hot path, are best read as noise. Together they cast doubt on the parent's earlier per-op improvement. This is the third cycle where small latency deltas looked unreliable.
- Removing a cached field from Node did not shrink it, because padding or alignment absorbed the saving. Moving the hash recompute onto the eviction path then taxed put tail latency. Node size has little room left to move this way.
- Cheaper sketch increment work (one hash, shift-derived indices, saturation early-exit) is the only speed idea this cycle that cleanly left hit-rate alone. It confirms the cycle-0002 finding that sketch cost is a main per-op term, now on an independent baseline lineage.
- Hit-rate stayed flat across the cohort. No variant touched a mechanism that could change it.
- Two insights were written without a source diff, so some attributions are inferred. v003 cannot separate the early-exit from the cheaper hashing.

## Open questions

- Does v003's increment speedup stack on the cycle-0004 v001 parent (cheap map hasher, MRU skip, compact links), or is it only visible against the baseline?
- Which part of v003 matters: the saturation early-exit or the single-hash derivation?
- Does any footprint lever remain once Node alignment is the floor, short of changing counter or key types?
- Is there any hit-rate lever left on workload-a?

## Continuation

Use v003's single-hash, shift-derived increment as the speed component and merge it onto the cycle-0004 v001 lineage; test the early-exit and the hash derivation separately. Drop sketch width tuning, and drop Node field removal unless it is paired with a layout change that crosses an alignment boundary. Repeat latency runs before crediting any small delta. Hit-rate gains still need a genuinely new mechanism.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
