# Variant v002 Insight

## Hurt

- **hit-rate**: Repartitioning to 256 smaller buckets degraded hit-rate from 0.6611 to 0.6568, contradicting the hypothesis that narrower buckets would isolate hot items from collision-induced eviction.
- **per-op-time**: Get latency increased from 69ns to 100ns (p50), suggesting the cost of searching within a more fragmented bucket space outweighed any cache locality benefits.

## Helped

- (None observed across dimensions that improve overall cache utility.)

## Takeaway

Doubling bucket count while halving bucket size on a Zipfian workload with ClassicLRU eviction does not isolate hot items; instead, it increases search cost and reduces hit-rate. The hypothesis that collision isolation drives improvement fails in practice, likely because the eviction policy already handles mixed-temperature items adequately within wider buckets, or because the search cost of more buckets negates any isolation gain.
