# Variant v002 Insight

## Helped

_None. Hit-rate did not move from the parent, and no other dimension was measured for this variant._

## Hurt

_None measured. The fitness vector for v002 contains only `hit-rate`, so `metadata-size` and `per-op-time` cannot be judged against the parent from this evaluation._

## Takeaway

H02 added an `AgedLfu` policy that halves every item's `lfu_counter` once per 64 bucket hits, aiming to let stale-but-once-hot items decay. On workload-a at size 2048, the hit rate is identical to the ClassicLRU baseline, so the decay did not change which items the cache kept on this trace. Either the eval path does not select the new policy, or the fixed 64-hit period never shifts the eviction order enough to matter at this bucket geometry; the eval configuration would need to confirm which before the aging mechanism is judged.
