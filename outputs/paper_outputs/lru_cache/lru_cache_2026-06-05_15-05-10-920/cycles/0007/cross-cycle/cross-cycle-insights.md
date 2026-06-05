# Cross-Cycle Insights

## Recurring wins

- **Frequency-based protection consistently improves hit rate.** Every winning variant protects frequently-accessed items from eviction. The strategy evolved from static counters (Cycle 0001) to decay-aware mechanisms (Cycle 0002+), but the core insight—that workload access patterns are skewed and protecting hot items drives gains—has held across all cycles.

- **Multiplicative decay with fixed intervals outperforms alternatives.** The 0.75× multiplicative decay at capacity × 16 intervals emerged as robustly tuned; both gentler (0.8×) and more aggressive (0.7×) coefficients regressed (Cycle 0005), and deviations in interval (capacity × 8 or × 24) consistently hurt (Cycles 0004, 0006). Fixed simplicity beats adaptive complexity or curve-shape variations.

- **Unified frequency-based competition beats partitioned strategies.** Every zone-based attempt (hot/cold splits, resized partitions, selective decay) has regressed or underperformed (Cycles 0002, 0003, 0007). The cache benefits from all items competing fairly on frequency; artificial promotion barriers obscure frequency signals that matter across the entire cache.

- **Temporal locality within equal-frequency tiers matters.** Two-pass LRU victim selection (Cycle 0006 v002), which finds minimum frequency once then selects the head-most item, became the winner and remained stable (Cycle 0007 v001) despite marginal deltas. This refinement respects linked-list semantics and captures real structure in the workload.

## Recurring failures

- **Zone-based partitioning consistently regresses.** Cold/hot splits, hybrid LRU+frequency zones, and resized hot zones (Cycles 0002, 0003) all degraded performance. Partitioning forces items to compete within artificial boundaries rather than on true frequency merit, allowing moderately-frequent items to be evicted before demonstrating persistent reuse value.

- **Adaptive decay and threshold-based mechanics break discrimination.** Attempts to adjust decay dynamically (Cycle 0003 v002), accelerate adaptation (Cycle 0004 v002), or selectively preserve weak frequency signals (Cycle 0007 v002) all regressed. Uniform decay at fixed intervals is the robust pattern; workload phase shifts are gradual, not abrupt, and the fixed interval balances accumulation and adaptation correctly.

- **Non-multiplicative decay curves introduce complexity without gain.** Exponential approximation (Cycle 0005 v003) and right-shift division (Cycle 0004 baseline) both underperform gentler multiplicative forgetting. The 0.75× coefficient preserves frequency discrimination across reset cycles better than alternatives.

- **Counter saturation and other micro-level constraints are not bottlenecks.** Capping frequency counters at 32 (Cycle 0004 v001) had no impact. The decay mechanism already refreshes priorities fast enough that artificial lower limits add no benefit.

## Open questions

- **Does frequency-recency hybrid scoring compound with the decay incumbent?** Cycle 0007 v003 showed promise (freq × 100 + position) on baseline, but was not tested against the full stack of decay + two-pass selection. Does hybrid eviction scoring deliver additive gains or conflict with the incumbent mechanism?

- **Has victim-selection refinement plateaued?** v001's neutral delta in Cycle 0007 despite winning suggests the optimization frontier may be shifting. Are further tie-breaking improvements exhausted, or are there unprobed heuristics (e.g., decay-adjusted scoring, frequency-quantization sharpening) worth testing?

- **Do the 0.75× coefficient and capacity × 16 interval generalize?** All testing occurred on a single trace. Sensitivity to cache capacity, workload diversity, and access-pattern structure remains unknown. Is this tuning portable or workload-specific?

- **What drives sensitivity to decay coefficient?** The tight plateau around 0.75× (Cycle 0005 showed failure in both directions) suggests the workload's frequency distribution or phase-shift structure is critical. Understanding whether this reflects multimodality or temporal phase shifts could guide future exploration.

## Per-policy notes

- **Frequency-based eviction (all cycles, incumbent).** Static frequency counters (Cycle 0001) provided the foundational insight. Periodic multiplicative decay (0.75× at capacity × 16, Cycles 0002–0007) refined this into a temporally adaptive strategy that protects hot items while gradually forgetting ancient hotspots. This policy is conclusively optimized on the decay dimension; further exploration should shift away from forgetting mechanics.

- **Victim selection precision (Cycle 0006+, stable).** The optimization frontier has shifted from "what signals matter" to "how to apply them." Two-pass LRU tie-breaking within equal-frequency tiers (Cycle 0006 v002, Cycle 0007 v001) captures temporal locality and remains the stable winner despite marginal deltas. The narrow margin suggests this dimension is approaching saturation.

- **Hybrid scoring (Cycle 0007 v003, unexplored orthogonal).** Frequency-recency weighted eviction (freq × 100 + LRU position) showed promise on baseline but has not been validated against the decay incumbent. This remains the strongest unexplored dimension for potential improvement; testing it within the full eviction stack (decay + two-pass) is the natural next step.
