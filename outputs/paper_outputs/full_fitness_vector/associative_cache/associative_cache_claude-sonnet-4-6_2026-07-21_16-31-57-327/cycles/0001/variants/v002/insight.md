# Variant v002 Insight

## Helped

- hit-rate: Widening buckets from 16 to 64 slots reduced premature eviction of hot entries on the LIRS workload-a trace, where the hot working set fits comfortably within the larger per-bucket capacity.

## Hurt

- per-op-time: The larger bucket size forces `get` operations to scan up to 4× more entries per bucket on a miss or LRU traversal, pushing median get latency notably higher than the baseline.

## Takeaway

Repartitioning from 128×16 to 32×64 confirms that the baseline's narrow buckets were a meaningful constraint on hit rate for skewed LIRS access patterns — the working set fits within wider buckets and survives longer. However, the same wider buckets impose a per-lookup scan cost: get latency regressed significantly even as hit rate improved, indicating that the hit-rate gain comes with a real throughput trade-off that any future geometry change must account for.
