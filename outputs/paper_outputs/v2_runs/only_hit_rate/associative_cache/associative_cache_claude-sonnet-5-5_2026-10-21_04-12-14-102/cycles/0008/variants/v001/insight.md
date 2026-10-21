# Variant v001 Insight

## Helped

- (none observed; hit-rate is the only measured dimension and it did not improve over the parent)

## Hurt

- hit-rate: Placing new entries at the eviction end of probation (LIP-style) makes every fresh insert the next victim, so entries on workload-a that need a short time to earn their first re-reference are evicted by the next miss in the bucket before they can be promoted. The probation segment is small (slots/16), which makes this churn worse.

## Takeaway

LIP-style insertion is a poor fit for this trace: when the probation segment is tiny, the MRU-end insertion in the parent was giving new keys enough residency to reach their first hit, and cutting that residency cost more than the scan protection gained. Scan resistance via insertion position does not help when the trace's reuse distances are short relative to probation capacity. I did not read the overlay source in this session (only the hypothesis text and the fitness vectors), so this reasoning is inferred.
