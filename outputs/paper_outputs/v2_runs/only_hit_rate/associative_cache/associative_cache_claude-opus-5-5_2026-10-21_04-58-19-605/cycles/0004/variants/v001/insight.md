# Variant v001 Insight

## Helped

(None. Hit-rate is the only dimension recorded, and it went down.)

## Hurt

- hit-rate: In a full bucket, new keys now go in at index 0 instead of index 1. That cuts probation from about two misses in the bucket to exactly one: the bucket's next miss evicts the newcomer unless it was hit first. Keys whose first reuse comes after two or more misses in their bucket are now evicted before that reuse. On workload-a, this costs more than the parent's single buffer slot cost by keeping one-off keys around. (Per-op-time was not recorded. Each miss now runs `Vec::insert(0, …)`, which shifts the whole bucket instead of all but one element, so any miss-path cost is likely small and is unconfirmed.)

## Takeaway

Going from index 1 to pure LIP reversed the trend from the earlier depth steps. On workload-a, a new key needs a grace period of at least one more miss in its bucket than strict LIP allows. That makes the parent's 1/8 depth, which in a full bucket means index 1, the best depth tested on this line. This also settles H01's bracketing question: the optimum is not at the eviction end. Further gains are more likely to come from changing how newcomers earn protection than from moving the insertion point.
