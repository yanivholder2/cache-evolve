# Variant v002 Insight

## Helped

## Hurt

## Takeaway

Placing newly inserted keys at the LRU tail, with hit path and eviction unchanged, left hit-rate on workload-a at size 2048 exactly equal to the baseline, so this change did not move any measured dimension. That result is a null for the scan-resistance hypothesis on this trace; it also means the overlay's effect on this workload should be verified before treating the match as evidence, since an identical value to four decimal places can indicate the insert-position change never affected evictions here.
