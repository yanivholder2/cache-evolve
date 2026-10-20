# Cross-Cycle Insights

## Recurring wins

- **Geometry concentration as the primary performance lever**: From cycle 0001 baseline (25/75) through cycle 0004 (85/15), steadily increasing hash-indexed capacity fraction delivered sustained improvements in both hit-rate and latency on Zipfian workloads. The progression held across all tested configurations until saturation point identified in cycle 0005.

- **Eliminating unnecessary hot-path work**: All successful variants—back-insert promotion filtering, geometry rebalancing, frequency-gated promotion—reduced cache churn or traversal work. Variants that succeed tend to prevent wasteful tier transitions or access cascades, simultaneously improving hit-rate and latency.

- **Adaptive parameter scaling unlocks composition**: Cycle 0005 demonstrated that frequency-gating selectivity must scale with geometry allocation. The adaptive threshold computation (`max(1, stash_buckets / 4)`) made promotion gating orthogonal to bucket geometry, enabling clean composition where hardcoded tuning had failed. This pattern—deriving parameter values from allocation ratios rather than treating them as independent constants—appears to be the key to reliable cross-axis optimization.

- **Simple orthogonal improvements outperform complex composition**: When multiple strategies succeed in a cohort, geometry-only variants consistently deliver the cleanest wins (lowest trade-off burden) relative to multi-axis compositions with identical metric improvements.

## Recurring failures

- **Back-insert promotion**: Consistently failed or regressed latency across all tested geometries (cycles 0001–0004). The MRU-insertion mechanism carries a structural cost that no amount of primary-tier capacity expansion can overcome. Dropped after cycle 0004.

- **Hardcoded threshold tuning without geometry adaptation**: Attempts to refine frequency-gating thresholds (e.g., lowering from 2 to 1) without adjusting for stash tier capacity caused selectivity mismatches. Threshold=1 on 75/25 geometry flooded the tier with false-positives, degrading hit-rate despite earlier success on concentrated geometries. This signaled the need for adaptive, not manual, threshold computation.

- **Pushing geometry concentration past saturation**: Cycle 0005 revealed that 90/10 geometry improves latency while regressing hit-rate as stash capacity drops below adequacy for the workload's tail. On Zipfian distributions, capacity utilization (hit-rate preservation) outweighs operation-level speed gains, making the 85/15–90/10 boundary a hard asymptote.

## Open questions

- Is there an optimal geometry allocation between 75/25 and 85/15 (e.g., 80/20 or 82/18) that better balances hit-rate and latency than the cycle-0005 endpoints?

- Can the adaptive threshold scaling formula itself be refined (e.g., `stash_buckets / 3` or `stash_buckets / 5`) to shift the trade-off frontier, or is division-by-4 near-universal?

- Does adaptive threshold on aggressive geometries within the sweet-spot range (80/20–85/15) generalize cycle-0005's composition success and preserve hit-rate while improving latency?

- Are there orthogonal optimization axes beyond bucket geometry and promotion gating (e.g., eviction ordering, access-pattern-specific tiering, or cache-aware struct layout refinements) that could yield additional gains on top of aggressive geometry?

## Per-policy notes

- All cycles tested variations of the same bucket-tiered eviction policy (hash-indexed primary tier, stash secondary tier, frequency-gated promotion between tiers).

- Workload is consistently Zipfian (small set of hot keys dominates). Improvements tracked both dimensions simultaneously under this distribution; hit-rate remains harder to preserve than latency when geometry concentrates beyond the optimal range.

- Cycle 0001's structural insights (three independent optimization layers, reduction of unnecessary work) held across all subsequent cycles; they correctly predicted the space's decomposition into orthogonal dimensions (geometry allocation, promotion filtering, struct packing).

- Geometry concentration demonstrated no saturation until cycle 0005, suggesting the baseline left substantial room on the allocation axis. The eventual asymptote (85/15–90/10 range) confirms concentration is not a bottomless lever but remains the most reliable single axis for improvement across diverse parameter combinations.
