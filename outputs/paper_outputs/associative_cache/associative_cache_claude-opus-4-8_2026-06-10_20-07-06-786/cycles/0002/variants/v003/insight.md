# Variant v003 Insight

## Helped

- hit-rate: Reusing `lfu_counter` as a probationary/protected flag and evicting the front-most probationary item first means one-hit-wonders churn through the front segment while keys proven by a second access are shielded, so the protected working set survives scan bursts that ClassicLRU's single-segment recency would have flushed.

## Hurt

- per-op-time: `eviction_candidate_index` now linearly scans the bucket for the first probationary item instead of the baseline's O(1) index-0 victim, adding a per-eviction cost proportional to bucket size (not re-measured in this variant's vector, but a structural regression of the change).
- metadata-size: Unchanged — promotion state is carried in the existing `lfu_counter` field with no new per-item storage, so segmentation comes for free on the memory dimension.

## Takeaway

Partitioning a bucket into a promote-on-second-hit SLRU, with no new metadata, beats single-position LRU admission on this workload: demanding proof of reuse before granting protection is what raises hit-rate, and the only cost is a linear victim scan rather than any extra memory.
