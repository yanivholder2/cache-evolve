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
