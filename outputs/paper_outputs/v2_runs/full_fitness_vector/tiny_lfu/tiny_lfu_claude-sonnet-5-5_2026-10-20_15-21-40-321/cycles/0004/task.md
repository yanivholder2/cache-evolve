# Task: cycle-insight-writer

Cycle: 0004
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0004

## Winners

- v003
- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0004/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0004/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0004/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0004/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0004/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0004/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three single-axis changes to the baseline were tried: sketch aging (v001), a small admission window (v002), and a sketch rewrite with a single hash, u8 counters and conservative update (v003). Only v003 won. It improved speed and sketch footprint with a small hit-rate gain.

## Patterns

- The two changes that added mechanism to the admission path (aging, window) gained nothing on hit rate and cost per-op time and struct metadata. The change that made the existing sketch cheaper and more accurate was the only one that paid off.
- Hit-rate hypotheses premised on "stale frequencies" or "newcomers need a trial period" did not hold on workload-a. The baseline's never-decaying counters and immediate strict-greater admission looked well suited to this trace.
- The baseline's per-op cost is dominated by repeated hashing in the sketch. Removing it gave the clearest gain, especially in the put tail.
- Added fields or bookkeeping on the hot path (halving counters, segment lookup on get) raised latency without a matching benefit.
- The metadata-size summary tracks Node size, not sketch heap, so v003's real 4x sketch saving was invisible in the headline number.

## Open questions

- Do the u8 saturation cap and conservative update still hold up on traces with larger frequency gaps or drift?
- Would aging help with a much longer period, or on a trace with real frequency drift? Would a larger window help?
- Which of v003's parts (single hash, u8 counters, conservative update) account for the gains?

## Continuation

Build on v003 as the new parent and push further on per-op cost and footprint (cheaper hashing, tighter sketch layout, less work on the hit path). Drop the naive aggressive aging and the ~1% window from v001 and v002. Retry them only in a materially different form, such as a much larger window, conservative aging or a doorkeeper, and only with a rationale tied to a specific trace property. Prefer hypotheses whose gains show in the headline dimensions.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
