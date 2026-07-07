# Variant v003 Insight

## Helped

- hit-rate: Doubling bucket capacity from 16 to 32 slots gives the cold-first selector a larger pool of `counter == 0` candidates per bucket, reducing fallback to LRU-front eviction and preserving more warm entries that were pushed away from the MRU tail by mid-point insertion.

## Hurt

_(no measured dimension declined)_

## Takeaway

Adding factor-2 geometry widening on top of mid-point insertion and cold-first eviction closes — and marginally exceeds — the hit-rate of the factor-4 pure-LRU configuration from cycle/0003/v001, showing the two mechanisms are additive rather than competing at this intermediate geometry. However, the improvement over the parent is modest, consistent with mid-point insertion being the dominant cold-concentrator: once buckets are wide enough to give the midpoint selector a meaningful buffer zone between new arrivals and the eviction front (~32 slots), the marginal gain from further widening is likely exhausted for this workload.
