# Variant v002 Insight

## Helped

- **per-op-time (get)**: Direct hash-indexing in the normal tier bypasses stash probing; allocating 90% of buckets to the normal tier increases the proportion of operations that take the faster indexed path rather than linear stash probing.

- **per-op-time (put)**: Higher normal-bucket density reduces the frequency with which insertions must probe the stash, lowering amortized put cost even though the reduced stash capacity increases insertion competition.

## Hurt

- **hit-rate**: Reducing the stash from 25% (8 buckets on 32 total) to 10% (3 buckets) cuts the capacity available for items that miss the normal tier, causing more evictions on Zipfian workloads where the working-set tail spills beyond the hash-indexed region.

## Takeaway

The 90/10 geometry exposes the asymptote that eluded the progression through 75/25. Latency improvements from increased hash-indexing are outweighed by hit-rate loss from insufficient stash capacity, revealing that 75/25 represents the optimal trade-off point—pushing further toward pure normal-tier allocation optimizes for latency at the cost of the more valuable hit-rate dimension on this workload.
