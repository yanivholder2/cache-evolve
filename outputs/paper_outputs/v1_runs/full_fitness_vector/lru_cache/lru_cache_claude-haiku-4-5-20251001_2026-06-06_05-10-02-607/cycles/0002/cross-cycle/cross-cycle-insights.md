# Cross-Cycle Insights

## Recurring wins

- **Data structure choice dominates latency predictability**: Across both cycles, implementation efficiency outweighs eviction strategy sophistication. Cycle 0001's linear-scan selective eviction (V002, V003) spiked p99 latencies despite better hit rates; Cycle 0002 showed that bounded search windows and frequency bucket indexing could recover both hit rate *and* latency when the data structure was optimized. The pattern is consistent: architectural bottlenecks matter more than algorithmic selectivity.

- **Frequency-aware eviction gains hit rate when indexed efficiently**: V002's frequency tracking improved hit rate in both cycles (stronger in 0001, +2.7% in 0002 at 0.6904 vs 0.6721). The improvement persists once the O(N) scan is removed, suggesting the signal is real and separable from implementation overhead.

- **Structural partitioning provides latency robustness**: Bounded search windows (Cycle 0002 V001) and zone pre-partitioning (Cycle 0001) both reduce tail latency variance by constraining the eviction search space, even without complex metadata.

## Recurring failures

- **Linear eviction scans are a fundamental bottleneck**: Cycle 0001's V002 and V003 both suffered severe put p99 spikes (20× and 2× respectively) due to O(N) candidate scanning; Cycle 0002's V003 repeated the failure with a bounded window scanning strategy that still proved too expensive. Any eviction strategy that requires iterating a large candidate pool, even with protective marks, will exhibit unpredictable latency.

- **Metadata overhead without efficient indexing does not pay off**: V003 consistently underperforms (modest hit rate gains in 0001, failure in 0002) despite adding resurrection tracking or zone+frequency complexity. The pattern suggests that selective eviction signals (frequency, recency, resurrection history) require structured indexing to justify their cost; unindexed metadata becomes pure overhead.

## Open questions

- **Does V001's bounded window maintain hit rate on non-Zipfian or adversarial access patterns?** Both cycles tested synthetic or Zipfian-like distributions. Real workloads may be less concentrated, making the first min(capacity/4, 256) nodes unrepresentative of hot items.

- **Can V002's frequency bucket overhead be reduced on the fast path?** Cycle 0002 showed get p99 and put p50 regressions (67→147 ns and 63→70 ns) due to bucket management. Adaptive thresholds, lazy updates, or approximate bucketing might recover the hit rate gains without pointer-chasing cost.

- **How do the Winners scale under cache saturation and working-set instability?** Both cycles assumed stable or partially saturated caches. Sudden access pattern shifts or full saturation may expose weaknesses in V001's fixed window or V002's frequency decay.

## Per-policy notes

- **V001 (Spatial partitioning / Bounded search)**: Simple, latency-predictable (3.2× put p99 improvement in Cycle 0002), zero metadata overhead. Risk: hit rate may degrade on non-concentrated workloads where the bounded window misses hot items outside the first window.

- **V002 (Frequency-recency hybrid / Bucket indexing)**: Strongest hit rate gains (0.6904 in Cycle 0002, superior in Cycle 0001), resolves Cycle 0001's latency spike with bucket indexing (15.6× put p99 improvement). Trade-off: measurable normal-case overhead (get p99 +219%, put p50 +11%) due to bucket complexity. Merits continued optimization.

- **V003 (Resurrection count / Zone+frequency hybrid)**: Consistently marginal or negative. Metadata signal (resurrection count, zone+frequency) does not compensate for scanning inefficiency in Cycle 0001 or added complexity in Cycle 0002. Not a viable path forward without fundamental restructuring.
