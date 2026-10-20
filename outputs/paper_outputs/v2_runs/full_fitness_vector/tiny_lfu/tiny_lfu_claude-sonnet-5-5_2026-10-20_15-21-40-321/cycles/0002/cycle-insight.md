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
