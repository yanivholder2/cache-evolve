# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0005

## Winners

- v002
- v001
- v003

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0005/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0005/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0005/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0005/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0005/hypotheses/H03.md

## Prior cycles digest

### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Three variants were built on the cycle-0001 sketch: a single-mix hash with bit-sliced indexes and a cached hash (v001), a long-period counter halving (v002), and a 4-bit nibble-packed single-hash sketch (v003). All three were kept as Winners. Their gains were uneven: v001 was a clean speed win, v003 traded hit-rate for speed and footprint, and v002 gained little and lost hit-rate.

## Patterns

- Hashing and index derivation remain the main source of per-op cost. Both v001 and v003 cut it, and the put tail improved the most.
- Shrinking sketch precision (4-bit counters, one row, tight sizing) buys speed and footprint, but hit-rate falls noticeably. Changes that left counters, conservative update and admission untouched kept hit-rate essentially flat.
- Aging failed again, even with a period 10x longer. workload-a has stable frequencies, so decay discards useful history. Two cycles of evidence now point the same way.
- Every variant that added state to the cache struct grew it slightly. The headline metadata number tracks Node size, so these changes and the sketch heap savings do not show up there.
- v001's insight was written without access to the source, so its attributions (hash caching versus the mixing change) are inferred, not confirmed.

## Open questions

- How much hit-rate does v003's nibble layout recover with a second hash or a larger table, and at what speed cost?
- Is v003's loss mainly from the 4-bit cap, the single row, or the tight sizing?
- Does v001's speedup combine with v003's compactness, or does the cached-hash state offset the footprint gain?
- Is there any trace property that would justify decay? Without one, aging has no case.

## Continuation

Treat v001 as the preferred parent for speed: it has no hit-rate cost. Next, explore the v003 compact layout combined with v001's hashing, adding the minimum extra accuracy (a second hash or a larger table) needed to recover hit-rate. Drop periodic halving and any other aging variant unless there is a drifting trace to justify it. Target admission quality or sketch accuracy for hit-rate gains, and prefer hypotheses that move the headline dimensions.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
