# Variant v002 Insight

## Helped

- **hit-rate**: Expanding the normal tier from 25% to 85% of total buckets increases reliable capture of cold misses in the hash-indexed primary tier, improving overall hit-rate by 7.3 percentage points.
- **per-op-time**: The larger normal bucket allocation (27 vs. 8 buckets) reduces the frequency of stash lookups and demotions, shortening probe depth and lowering latency across all percentiles (get p50: 183→111 ns, put p50: 142→115 ns).

## Hurt

(None observed)

## Takeaway

Pushing the normal/stash allocation from baseline's conservative 25/75 ratio to 85/15 continues to yield clean wins in both hit-rate and latency, confirming the asymptote remains open. The capacity shift from a small stash (8 buckets) to a substantially larger normal tier (27 buckets) trades occasional demotions for dramatically faster cold-miss handling in the primary hash index. The hypothesis's expectation that this pure-geometry extension would outperform complex variants is validated.
