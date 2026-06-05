# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Shrinking the stash from 4 to 2 slots (15/16 split) eliminated the cold-burst buffering headroom that the parent's 4-slot stash provided, and the two additional normal-tier slots were insufficient to offset those extra evictions.

## Takeaway

Pushing the normal/stash split from 7/8 (28+4) to 15/16 (30+2) reversed the previous upward trend in hit-rate, pinning the inflection point between 28 and 30 normal buckets for this workload. At only 2 stash slots, cold-arrival bursts that the parent absorbed without harm now compete directly with hot-tier entries, and the inherited second-chance recovery cannot fully compensate for the lost stash headroom. The 7/8 split appears to be the effective ceiling for hot-tier expansion under the current eviction policy and workload.
