# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0008

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

The cohort tried a probation/protected split of the recency list (v001), a second-chance referenced-bit eviction in place of move-to-front (v002), and insertion of new keys at the LRU tail with only periodic MRU insertion (v003). Only v003 reports a hit-rate gain; v001 and v002 both report a hit-rate dip, yet v002 is listed as the Winner.

## Patterns

- The only hit-rate improvement came from changing where new keys enter the recency order (v003), not from adding structure. It is a scan-resistance effect that needs no second list or ghost set.
- Both changes that touch eviction order lowered hit-rate on workload-a at size 2048: the segmented recency list (v001) and the second-chance approximation (v002). Strict recency ordering appears to be doing useful work on this trace.
- The Winner list includes v002, but its own insight reports a hit-rate dip below its parent and no offsetting gain. Its selection rests on something the insight does not show, the same selection-criterion problem flagged in cycles 0003 and 0004.
- Per-op time and metadata size were not recorded for any variant this cycle. That gap now spans five cycles, so no lineage has evidence on the dimensions its changes were written to improve.

## Open questions

- What justifies v002 as a Winner when its hit-rate fell below its parent and no time or size gain was recorded?
- Does the LRU-tail insertion in v003 hold up on other traces, or is the gain specific to workload-a at size 2048? Its scan-resistance mechanism would only matter on scan-heavy traffic.
- Is the segmentation idea dead as written, or does the protected cap or demotion rule (the parameters v001 points to) carry real headroom?

## Continuation

Pursue v003's insertion-position change as the lead candidate: it is the only hit-rate gain this cycle and adds no structure. Drop the probation/protected split as written and drop second-chance eviction, since both lowered hit-rate. Re-check v002's Winner status before extending its lineage, and add per-op time and metadata size to the fitness vector before judging any lineage.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
