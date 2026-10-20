# Variant v002 Insight

## Helped

- **hit-rate**: Pushing the normal tier allocation to 62.5% (20 of 32 buckets) catches even more cold misses in the hash-indexed tier on first lookup, further reducing items that must traverse the unindexed stash.

- **per-op-time**: With more capacity in the indexed tier and fewer items requiring stash traversal, both get and put operations see reduced latency (get p50 120→117ns, get p99 211→208ns); the smaller stash (12 vs 16 buckets) is offset by the performance gain from avoiding unindexed lookups.

## Hurt

(none)

## Takeaway

Continuing to shift capacity from the unindexed stash toward the hash-indexed normal tier yields sustained improvements in both hit rate and operation latency. The allocation progression—from baseline's 25/75, to v002's 50/50, now to 62.5/37.5—shows that hash-indexed capacity is consistently more valuable than stash capacity for this workload, even as the stash shrinks. The optimization has not yet reached saturation, suggesting further concentration toward the normal tier may be possible, though diminishing returns and increased eviction risk become factors at more extreme ratios.
