# Cross-Cycle Insights

## Recurring wins

- **Geometry reallocation is the primary performance lever**: Sustained improvement across all six cycles (25/75 → 50/50 → 62.5/37.5 → 75/25 → 85/15), with optimal range now localized to 75/25–80/20. Hit-rate and latency both improve in tandem within this band; further concentration (90/10) saturates hit-rate as stash shrinks below workload tail requirements.

- **Adaptive parameter scaling unlocks orthogonal composition**: Starting in cycle 0005, shifting from hardcoded constants to initialization-time geometry-aware parameters (e.g., threshold = max(1, stash_buckets / 4)) makes frequency-gating selectivity adapt across all allocations. This eliminates the tight coupling between promotion logic and bucket allocation, enabling reliable composition.

- **Simple variants outperform complex composition**: When multiple strategies succeed (e.g., cycle 0006), pure-geometry approaches consistently deliver cleaner trade-off profiles than composition-based strategies. Geometry alone on 80/20 yields 7.5% hit-rate gain and 37% get-latency improvement with manageable put-latency cost.

- **Reducing cache churn improves both throughput and accuracy**: All successful variants eliminate unnecessary work on the hot path—false-positive tier transitions, cascading demotions, or redundant traversals. The workload rewards capacity preservation over operation-level acceleration.

## Recurring failures

- **Back-insert promotion is structurally expensive**: Consistent latency regressions across all tested geometries (50/50, 62.5/37.5, 75/25, 85/15). The MRU-insertion overhead cannot be absorbed by primary-tier capacity increases; this is not a tuning problem but a mechanical cost that cannot compose.

- **Hardcoded threshold tuning is geometry-specific, not universal**: Fixed frequency gates (e.g., threshold=1 vs. threshold=2) create divergent outcomes on different allocations—over-filtering on conservative geometry, under-filtering on aggressive geometry. Manual per-geometry calibration is fragile and does not scale.

- **Composition with opposing forces creates interference**: When both geometry and gating push aggressively (e.g., 85/15 + threshold=1), stash capacity becomes a bottleneck and promotion cascades regress both hit-rate and latency. Composition succeeds only when parameters align with the geometry's capacity envelope.

- **Geometry saturation is real**: The asymptote at 85/15–90/10 reveals that further concentration is not a bottomless lever. Shrinking stash below workload tail requirements causes hit-rate degradation that offsets latency gains.

## Open questions

- Does adaptive threshold (divisor 3) on the optimal 80/20 geometry compose without regression, combining hit-rate and latency gains from both pure geometry and gating refinement?

- Can the adaptive formula divisor itself be made geometry-aware (scaled by allocation ratio) to achieve universal optimality across all allocations, or does each geometry band require a different constant?

- What optimization axes remain beyond bucket geometry and promotion frequency? (e.g., eviction policies, bucket-size tuning, per-bucket counters, adaptive thresholds per tier, tier-crossing heuristics)

## Per-policy notes

- **Frequency-gated promotion with adaptive thresholds**: Effective when gate selectivity scales with stash capacity (e.g., divisor 3 on 25/75 yields threshold=8; divisor 3 on 80/20 yields threshold=2). Static thresholds create geometry-specific tuning burden and are not portable. Divisor 3 shows promise as a formula refinement over divisor 4.

- **Back-insert promotion (MRU ordering)**: Abandoned after cycle 0004. Consistently fails across all geometry configurations; the insertion overhead is fundamental, not tunable via capacity allocation.

- **Pure-geometry allocation (no gating)**: Most reliable, simplest to reason about. Current sweet spot is 80/20 (or range 75/25–80/20), delivering both hit-rate and latency improvements. Geometry-only variants should remain the default unless adaptive gating proves to reconcile all trade-offs on optimal geometry.
