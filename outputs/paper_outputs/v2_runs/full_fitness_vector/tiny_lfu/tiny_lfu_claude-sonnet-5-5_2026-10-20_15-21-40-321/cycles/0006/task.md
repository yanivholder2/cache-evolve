# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0006

## Winners

- v002
- v003
- v001

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0006/hypotheses/H03.md

## Prior cycles digest

### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

Three single-axis changes were tried: u32 list indices in place of `Option<usize>` links (v001), a two-candidate victim sample (v002), and a tie-admitting `>=` comparison (v003). v001 and v002 were kept as Winners. v003 was not. All three cost per-op time, and none gave a real hit-rate gain.

## Patterns

- The only change that moved a headline dimension was v001's layout shrink. It cut Node size with hit-rate unchanged, but it gave back per-op latency. The working set is small, so the locality benefit did not appear. Extra index conversion and sentinel checks seem to cost more than the saved cache lines.
- Both admission-quality changes (wider victim sample, looser tie rule) failed to lift hit-rate. The wider sample was marginal and probably noise. Admitting ties lowered hit-rate. Strict greater-than admission with the LRU head as victim again looks well suited to workload-a.
- Frequencies on this trace are low and often tied. Ties are common, so the tie rule matters, and loosening it removes protection for resident entries.
- Latency rose even where the hot path was untouched (v002's get, v003's one-operator edit). Some of the per-op regressions are probably measurement noise or code-layout effects, so small time deltas are unreliable evidence.
- Two of the insights were written without a source diff, so some of their attributions are inferred.
- A cheap victim-selection or comparison tweak is now ruled out as a hit-rate lever, adding to the earlier failures of aging and a window.

## Open questions

- Is v001's size gain worth its latency cost? Does the cost come from NIL checks and widening, or is it noise?
- Can the per-op regression be recovered, for example with unchecked indexing or a layout that avoids repeated conversion?
- Is there any signal that separates tied frequencies without adding hot-path work?
- How much of the observed latency variance is noise, and does it need repeat runs?

## Continuation

Keep v001 as the compact-layout parent, but aim next at winning back its per-op time (fewer conversions, tighter hot-path code) and combine it with cycle 0002's v001 hashing speedup and the compact sketch. Drop victim sampling and tie-admitting comparisons. Hit-rate gains need a genuinely new signal (a doorkeeper or sketch accuracy), not tweaks to the admission comparison or victim choice. Use a larger or repeated measurement before trusting small latency deltas.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
