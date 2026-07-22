# Cross-Cycle Insights

## Recurring wins

- **Baseline simplicity holds steady**: ClassicLRU's recency-only eviction maintained consistent hit-rates (0.6611) across test variants without introducing per-access overhead

## Recurring failures

- **Added semantics, zero policy gain**: Frequency tracking (LFU), bucket repartitioning, and Swap each increased latency (45% to 3× slowdowns) while hit-rate remained flat or worsened
- **Theory-practice inversion**: Asymptotic advantages (O(1) removal for Swap vs. ClassicLRU's O(n)) were overwhelmed by constant factors—cache coherency, memory latency, and bookkeeping overhead—at practical benchmark scale
- **Workload-semantic mismatch**: Zipfian access patterns appear fully captured by recency; frequency tracking and finer bucket isolation did not unlock additional cache efficiency

## Open questions

- Would batched or vectorized updates to bookkeeping structures recover the per-operation cost while preserving policy semantics (frequency awareness or O(1) removal)?
- Does ClassicLRU's dominance hold across non-Zipfian distributions (uniform, skewed, temporal patterns) or cache geometries?
- Could adaptive or hybrid policies dynamically select between recency, frequency, and other signals based on online-observed hit-rate or access-pattern drift?

## Per-policy notes

- **ClassicLRU**: Baseline remains robust; consistent hit-rate signals that recency-only eviction aligns tightly with Zipfian request patterns
- **LFU**: Frequency bookkeeping added measurable overhead; absence of hit-rate improvement suggests Zipfian workload does not reward frequency tracking
- **Bucket repartitioning**: Finer-grained isolation increased latency; no corresponding cache-efficiency gain, implying workload does not benefit from bucket segregation
- **Swap**: O(1) asymptotic removal advantage inverted under real memory effects; swap-chain overhead and cache-line bouncing negated theoretical gains at tested scale
