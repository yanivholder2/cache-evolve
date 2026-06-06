# Cross-Cycle Insights

## Recurring wins

- **Geometry concentration is the dominant, unblocked lever**: Across all four cycles, increasing hash-indexed (normal-tier) bucket allocation at the expense of stash capacity consistently improves both hit-rate and latency. Progression from baseline (25/75) through 50/50, 62.5/37.5, 75/25, to 85/15 shows sustained gains with no saturation observed yet. This single axis has delivered the most reliable and composable improvements.

- **Eliminating unnecessary hot-path work pays reliably**: All successful variants reduce redundant operations—whether through capacity reallocation (fewer stash traversals), cache-friendly layouts, or filtering false-positive tier transitions. Simple variants that focus on one dimension consistently outperform complex compositions when both succeed.

- **Zipfian workload structure drives all payoffs**: Every winning strategy exploits the skewed access pattern where a small set of hot keys dominates. Capacity allocation toward faster (hash-indexed) lookup aligns perfectly with this skew.

## Recurring failures

- **Back-insert promotion has structural overhead**: Strong in cycle 0001, but consistently failed in later cycles. Composition with 50/50 geometry (cycle 0002) incurred latency penalty despite hit-rate gain. Pure latency regression on 85/15 geometry (cycle 0004) suggests the MRU-insertion cost cannot be absorbed by increased capacity. Dropped as a viable composition partner.

- **Promotion gating selectivity depends on geometry tuning**: Frequency-gated promotion (counter ≥ 2) succeeded on concentrated geometry (62.5/37.5+) by filtering false-positives, but failed on balanced geometry (50/50) by filtering legitimate candidates. Even when it succeeded, gating introduced measurable latency overhead. The threshold is not generic across allocation profiles.

- **Composition of logically orthogonal optimizations is empirically unreliable**: Geometry reallocation and promotion logic, despite operating on distinct layers, interact in unpredictable ways. Simple single-axis variants deliver cleaner wins than combinations, suggesting the workload's optimization margin is primarily unlocked by one lever at a time.

## Open questions

- **What is the geometry asymptote?** Does concentration beyond 85/15 continue to improve both hit-rate and latency, or do diminishing returns finally emerge?

- **Can frequency thresholds be made adaptive?** Could the gate's selectivity be stabilized across different geometry allocations by scaling the threshold (e.g., threshold = max(2, floor(stash_buckets / 4))), enabling clean composition?

- **What optimization axes remain?** Beyond promotion ordering and bucket allocation, could alternative strategies (adaptive per-bucket counters, alternative eviction policies, cache-line tuning) unlock gains on top of aggressive geometry?

## Per-policy notes

- **Hash-indexed tiering rewards concentration on Zipfian workloads**: The consistent success of shifting capacity from stash to normal tier confirms that this tiering structure trades stash flexibility for hot-path throughput. The workload skew aligns perfectly with fast-path optimization.

- **Simple geometry beats complex tuning**: When geometry-only variants and composition variants both succeeded (e.g., cycle 0003 v003 vs v002), the simpler approach delivered cleaner outcomes across all percentiles. Suggests promotion-logic refinements face diminishing returns relative to capacity reallocation.

- **All testing assumes LRU-based tiering**: No alternative eviction policies were explored; conclusions are specific to this tier structure and may not generalize to other policies or workload distributions.
