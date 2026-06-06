# Variant v002 Insight

## Helped

## Hurt

## Takeaway

Replacing the strict tail-evict in `evict_main()` with a 3-item frequency tournament produced no measurable change in hit-rate on workload-a. This null result indicates that frequency-position inversions at the rear of main — the pathological case the hypothesis targeted — either occur too rarely on this trace to affect the resident set meaningfully, or the inversions that do occur are already resolved quickly enough by move-to-front recency ordering that the tournament has no net effect. The recency signal provided by position in main appears to be the dominant driver of eviction quality for this workload; adding a narrow frequency tournament does not complement it.
