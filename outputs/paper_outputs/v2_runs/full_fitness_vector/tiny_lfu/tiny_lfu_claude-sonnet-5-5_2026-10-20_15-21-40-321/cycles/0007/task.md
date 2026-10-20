# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0007

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

Three changes were tried on the compact-layout parent: a cheap map hasher with an MRU short-circuit on hit (v001), a one-bit doorkeeper in front of the sketch (v002), and a 4x wider sketch (v003). All three were kept as Winners. Only v001 was a real improvement: it cut per-op time and struct size with hit-rate unchanged. v002 and v003 moved hit-rate little or not at all.

## Patterns

- The map hasher, not link layout or the sketch, was the largest remaining latency cost. Swapping it recovered the per-op time lost to cycle 0003's compact layout and kept that layout's size gain.
- Both sketch-accuracy changes failed to yield a meaningful hit-rate signal. The doorkeeper slightly lowered hit-rate and widened the latency tail. Widening the sketch gave only a marginal gain at a large hidden heap cost. Sketch accuracy now looks saturated on workload-a.
- Adding state or hot-path work (a doorkeeper bitset) again raised tail latency and grew the struct. This repeats the pattern from cycles 0001 and 0002.
- v003's per-op improvement cannot be explained by its code change and is most likely noise. This is the second cycle where small latency deltas looked unreliable.
- Sketch heap changes stay invisible in the headline metadata number.
- Two insights were written without a source diff, so some attributions are inferred.

## Open questions

- Does v001's gain hold up under repeat measurement, and which part accounts for it: the hasher or the MRU skip?
- Does v001 combine with the compact nibble sketch and cycle 0002's hashing speedup without losing hit-rate?
- With sketch accuracy, victim choice, tie rules, aging and windows all exhausted, is any hit-rate lever left on workload-a?

## Continuation

Use v001 as the parent and keep stacking per-op cost and footprint reductions on it (the compact sketch and cheap hashing from earlier cycles). Drop the doorkeeper and wider-sketch directions, since sketch accuracy is saturated and the wider sketch only trades heap for a marginal gain. Hit-rate ideas need a different mechanism entirely. Repeat latency measurements before crediting any small delta.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
