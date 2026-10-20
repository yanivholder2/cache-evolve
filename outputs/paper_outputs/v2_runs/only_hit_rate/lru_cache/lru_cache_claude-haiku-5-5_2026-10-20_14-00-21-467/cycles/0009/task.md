# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0009

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cohort tried tail insertion of new keys with a sparse MRU insert, stacked on the parent's lookup collapse (v001), a stricter variant that promotes only on the second hit (v002), and a swap of the key index's hasher with eviction and recency left alone (v003). v001 is the only Winner, yet its own insight reports a hit-rate drop against its parent, and no variant recorded a per-op time measurement.

## Patterns

- Both tail-insertion variants (v001, v002) lost hit-rate on workload-a at size 2048. Tail insertion is the only mechanism this cohort tested that touches eviction order, and both of its forms moved hit-rate down.
- This does not match cycle 0005, where tail insertion was the only hit-rate gain. The same mechanism now reads as a loss, so the earlier gain either depends on the parent it was measured against or does not transfer to a stacked form. The cycles are not reconciled.
- v001 is named the Winner while its only measured dimension fell. The selection-criterion problem flagged in cycles 0003 through 0005 is still open.
- v003 (hasher swap) held hit-rate, as a placement-only change should, but per-op time was not recorded, so its one intended effect is untested. This is the same unmeasured per-op gap carried since cycle 0001.
- The H03 label now refers to a hasher swap here, a pre-sized index in cycle 0003, a per-op check in cycle 0002, and a segmentation change in cycle 0004. The hypothesis mapping is not stable across cycles and should be fixed before lineages cite it.

## Open questions

- Does tail insertion help on its own against the current parent? The cycle 0005 gain and this cycle's losses cannot be separated from the insights.
- Why is v001 the Winner when its hit-rate fell and no per-op or size gain was recorded?
- Does the hasher swap in v003 reduce per-op time at all?

## Continuation

Test tail insertion alone against the current parent before extending any stacked lineage, since the evidence for it now conflicts. Drop second-hit promotion (v002) as written. Re-check v001's Winner status before building on it, and record per-op time for v003 before judging the hasher swap. Pin down what H03 means so future cycles cite one definition.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

The cohort tried a ghost-set admission filter (v001), tail insertion of new keys (v002), and a frequency-aware eviction that picks the least-hit entry within a bounded four-entry tail window (v003). The Winner, v003, is the only variant that reports a hit-rate movement, and that movement is marginal.

## Patterns

- Only the frequency-aware tail-window eviction (v003) moved hit-rate, and it moved it slightly upward on workload-a at size 2048. Its gain is the one positive signal in the cohort, and its Winner status is consistent with that.
- The ghost-set admission filter (v001) left hit-rate unchanged. Its insight notes the fitness vector has no admission or rejection counts, so it cannot distinguish a filter that never engaged from one that engaged without effect.
- Tail insertion (v002) gave a hit-rate exactly equal to baseline. That is either a true null or a sign the insert-position change never affected evictions. This cycle's result conflicts with cycle 0005 (gain) and cycle 0006 (loss), so the tail-insertion evidence is not yet usable.
- v003 adds a per-entry hit counter whose memory cost is not in the fitness vector. Per-op time and metadata size were again not recorded, so the Winner's cost side is unknown. This is the same unmeasured gap carried since cycle 0001.

## Open questions

- Does the v003 gain survive on a second trace or a different cache size, or is it specific to workload-a at size 2048?
- Did the v002 tail-insertion overlay actually engage? Until a counter shows it did, its null cannot be read as evidence against the idea.
- Did the v001 admission counter ever reach the filter? If not, the ghost-set idea has not been tested at all.
- What do the per-entry counter and the tail-window scan cost in per-op time and metadata?

## Continuation

Pursue the bounded frequency-aware eviction (v003) as the lead lineage, but only after recording per-op time and counter memory and re-running it on a second trace or size. Verify engagement counters for the tail-insertion overlay (v002) and the ghost filter (v001) before ruling either idea in or out. Drop the ghost-set admission rule as written unless its counter shows it engaged.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

The cohort tried decaying the hit counters examined by the victim scan (v001), capping the per-entry hit counter at a small saturating width (v002, the Winner), and a count-min frequency gate on admission against the LRU tail (v003). None of the three reports a hit-rate gain; the Winner v002 is within noise of its parent, and its claimed benefit is a smaller per-node footprint that the fitness vector does not measure.

## Patterns

- No variant moved hit-rate upward. The two counter-based changes (v001, v002) and the admission gate (v003) all left workload-a at size 2048 flat or slightly down, so frequency signal did not translate into retained hits in this cohort.
- The Winner, v002, was selected for a footprint benefit that was never recorded. The same selection-criterion gap flagged in cycles 0003 through 0007 applies here again.
- None of the three insights confirms that its mechanism engaged. The overlay source could not be inspected, and v003's admission counts are absent, so flat results cannot yet be read as evidence against the ideas.
- Metadata size and per-op time were again missing from the fitness vectors. This is the same unmeasured gap carried since cycle 0001, and it blocks judging any change aimed at memory or time.
- The frequency signal itself keeps recurring across cycles (the bounded hit-window in cycle 0007, the counter changes here). Its effect on hit-rate has been small or null each time, so the open question is whether it helps on any trace beyond workload-a.

## Open questions

- Does the v002 counter cap reduce metadata size at all, and by how much? Without that number, the Winner has no measured benefit.
- Did the v003 admission gate ever reject a candidate? If it never engaged, the count-min idea remains untested.
- Does the v001 decay change hit-rate on a second trace or a size sweep, or is any effect hidden at this single point?

## Continuation

Re-check v002's Winner status before extending it: keep it only if metadata size is recorded and shows a real reduction with hit-rate held. Drop the count-min admission gate as written unless an engagement counter shows it rejected candidates. Pause the decay idea (v001) until it is tested on a second trace or size; do not build further on it from this cycle alone.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
