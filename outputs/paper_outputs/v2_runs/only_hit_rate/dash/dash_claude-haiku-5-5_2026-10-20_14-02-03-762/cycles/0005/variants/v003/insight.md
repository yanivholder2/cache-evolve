# Variant v003 Insight

## Helped

- None. hit-rate matched the parent exactly on lirs workload-a at size 2048, and no other dimension was reported for this variant.

## Hurt

- None measured. The variant's fitness vector reports only hit-rate, so metadata-size and per-op-time were not evaluated and cannot be compared to the parent.

## Takeaway

Moving items demoted from the probing tier to the LRU end of their normal-tier bucket left hit-rate on workload-a at 2048 unchanged to four decimals. That suggests demoted items are rarely the first eviction candidate at this trace and size, or that the bucket-ordering change never alters a victim choice here. This Insight could not confirm which, because the overlay source was not readable during this run, so the diff itself was not inspected.
