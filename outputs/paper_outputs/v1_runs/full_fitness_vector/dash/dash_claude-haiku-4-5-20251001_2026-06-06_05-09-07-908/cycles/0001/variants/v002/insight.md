# Variant v002 Insight

## Helped

- **hit-rate**: Widening the normal bucket tier from 8 to 16 buckets means cold misses are more likely to land in their target bucket on first hash lookup, reducing the proportion of items that fall through to the stash.

- **per-op-time**: More items hitting the primary bucket directly rather than requiring stash traversal significantly reduces operation latency; unindexed stash lookups are slower than direct bucket access, so shifting capacity toward the indexed tier improves p50 and p99 latencies for both get and put.

## Hurt

(none)

## Takeaway

Rebalancing bucket allocation from 75% stash (baseline: 8 normal + 24 stash) to 50% stash (variant: 16 normal + 16 stash) improves both hit-rate and operation latency. The result shows that hash-indexed capacity is more valuable than unindexed stash capacity for this workload—even with a smaller stash, avoiding expensive traversals through the unindexed tier yields better throughput and higher hit rates.
