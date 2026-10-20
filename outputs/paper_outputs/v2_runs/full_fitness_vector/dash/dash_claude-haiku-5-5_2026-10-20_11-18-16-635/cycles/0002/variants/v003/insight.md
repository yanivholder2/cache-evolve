# Variant v003 Insight

## Helped

- hit-rate: Choosing the lowest-counter slot in `evict_one()` instead of the baseline LRU-only victim keeps frequently re-read residents from being displaced by one-shot inserts, which is the failure mode H03 targets.
- per-op-time: The per-slot access counter and the decay step only touch the get and eviction paths, and the faster get and put timings are consistent with the baseline's displacement-and-reinsert churn being reduced under this workload.

## Hurt

(none measured)

## Takeaway

Moving the protection from insert position into victim selection, using a 2-bit saturating counter that fits in existing slot padding, improved hit rate on workload-a without growing per-slot metadata. The per-op-time gain is larger than a victim-only change would normally explain, so it should be re-measured before it is treated as a settled effect of the counter.
