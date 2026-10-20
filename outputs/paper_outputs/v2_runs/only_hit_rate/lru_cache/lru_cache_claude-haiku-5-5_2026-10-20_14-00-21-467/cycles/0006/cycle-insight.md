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
