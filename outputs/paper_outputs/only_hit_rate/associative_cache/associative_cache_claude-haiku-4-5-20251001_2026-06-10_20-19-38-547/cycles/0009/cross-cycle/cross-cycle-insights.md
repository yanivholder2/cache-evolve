# Cross-Cycle Insights

## Recurring wins

- **Post-eviction recovery via bounded stash buffer**: From Cycle 0001 onward, capturing evicted items in a recovery buffer consistently outperforms redistribution, filtering, or geometry-only approaches. The workload exhibits temporal re-use that makes preservation worthwhile; the challenge is sizing and organization, not whether to preserve.

- **Two-tier stash organization with promotion-on-hit**: Introduced in Cycle 0006 and validated through Cycle 0009, the dual hot/cold tier mechanism with immediate promotion on re-access outperforms filtering-based insertion controls and staged gating across all test variants. Organization of captured items matters more than filtering of what to capture.

- **Asymmetric partition allocation reflecting workload shape**: Cycles 0007–0009 show that uneven tier sizing (18/7, 20/5) tailored to skewed re-access distribution outperforms symmetric partitioning. The workload's evicted items have predictable, concentrated re-use patterns; allocating more capacity to hot tier where these items cluster improves hit rate without added complexity.

- **Geometry and stash are complementary, not redundant**: Cycle 0004 proved that wider-bucket geometry (collision reduction) compounds with stash capacity, achieving joint gains neither axis provides alone. After collision-driven evictions are reduced geometrically, the stash captures remaining recoveries; both dimensions are binding constraints.

## Recurring failures

- **Filtering at insertion (any signal variant)**: Cycles 0003, 0005, and 0006 tested occupancy-aware, collision-aware, and access-count insertion filtering; all failed or regressed. The workload provides no signal that reliably predicts recovery value; items evicted from low-contention buckets are still worth preserving. Blanket capture at sized capacity works; filtering does not.

- **Counter-level phase response**: Cycles 0006–0009 repeatedly attempted to couple phase-detection (correctly identifying workload transitions) with counter manipulation (boosting/resetting item counters). All variants (counter reset, counter boost, counter amplification) failed to improve hit rate despite correct diagnosis. Phase detection works; phase *response* via item-level counters does not.

- **Staged/gatekeeping promotion**: Cycle 0008 (v002) required two re-accesses for hot-tier promotion; Cycle 0006–0007 variants with access-count filtering all regressed or null-resulted. Delaying promotion to hot tier trades hit-rate for reduced hot-tier churn, but the workload benefits from aggressive escalation. Items showing single re-access should immediately reach hot tier.

- **Fine-grained geometric tuning beyond 0.71**: Cycles 0005–0006 tested 0.70, 0.75, and fractional refinements; all regressed or null-resulted from the 0.71 baseline. Collision-reduction optimization converged sharply; further geometric tuning is exhausted.

## Open questions

- **Phase-aware dynamic partition rebalancing**: Cycle 0009 (v003) crashed when attempting to rebalance partition ratios during phase transitions. The concept (detect workload phase, adapt tier allocation dynamically) remains untested; can it succeed with simpler integration (e.g., forward-looking allocation only, no re-partitioning of in-stash items)?

- **Organizational strategies orthogonal to two-tier partitioning**: Cycles 0006–0009 matured two-tier promotion-on-hit and asymmetric allocation, but gains are now marginal (+0.5% for LRU in Cycle 0009). Do frequency-based aging, multi-level tiering without gatekeeping, or predictive stash prefetching unlock larger improvements than partition or within-tier refinement?

- **Has architectural ceiling been reached?**: After nine cycles, each new axis yields smaller gains (1.2% → 0.26% → 0.5%). Are there fundamentally different optimization levers (workload-adaptive eviction policies beyond FIFO, alternative bucket-layout strategies, or cache-replacement algorithms orthogonal to collision-reduction + stash recovery), or has this workload-cache pairing saturated?

## Per-policy notes

- **FIFO baseline with two-tier promotion-on-hit (20% hot / 5% cold)**: Dominant design across Cycles 0006–0009; proven across all partition ratios and most refinement attempts. Simple, stable, and effective at capturing skewed re-use patterns.

- **LRU within hot tier (Cycle 0009)**: Secondary refinement providing +0.5% gain over FIFO. Temporal recency of individual item access within the hot tier influences eviction ordering; gain is modest, suggesting the two-tier organization already captures most temporal locality.

- **Counter-free phase adaptation**: Phase detection (collision-rate divergence) works diagnostically, but counter manipulation fails operationally. If phase-aware dynamic response is pursued, decouple detection from counter-level item manipulation and instead couple to structural adaptation (partition rebalancing, geometry rebalancing during transitions).
