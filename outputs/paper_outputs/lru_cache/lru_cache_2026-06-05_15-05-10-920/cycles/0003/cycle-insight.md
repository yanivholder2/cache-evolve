# Cycle 0003 Insight

## Cohort outcome

This cycle tested three refinements to the Cycle 0002 winner (periodic frequency decay with fixed `capacity * 16` interval): zone-based partitioning with independent decay rates (v001), adaptive frequency-distribution-aware decay thresholds (v002), and expanded hot-zone size (v003). All three regressed relative to the parent strategy. The "winner" v002 represents the least degradation, suggesting the baseline approach has already converged to a well-tuned point and that orthogonal modifications introduce overhead without corresponding benefit.

## Patterns

- **Zone partitioning consistently backfires.** Both v001 (cold/hot split with LRU in cold zone) and v003 (resized hot zone) hurt hit-rate. The parent's unified frequency-decay structure allows all items to compete fairly on frequency; introducing zone boundaries creates artificial promotion barriers and masks frequency signals that matter across the entire cache. Items with moderate frequency get evicted from the cold zone before demonstrating enough reuse to earn hot-zone protection.

- **Adaptive complexity underperforms fixed simplicity.** v002's attempt to dynamically adjust decay timing based on observed frequency distribution added tracking and per-operation computation overhead without yielding gains. The fixed `capacity * 16` interval—simpler and lighter-weight—remains superior, suggesting either this workload lacks sharp phase transitions requiring real-time adaptation, or the cost of tracking averages simply outweighs phase-shift benefits.

- **The parent strategy is robustly tuned.** The fact that all three orthogonal modifications regressed suggests the Cycle 0002 winner has already captured the major structural advantage (protecting hot items via frequency decay while gradually forgetting ancient hotspots). The narrow window for further improvement indicates this cache is near a local optimum for this workload.

## Open questions

- Is the workload fundamentally lacking the structural properties (e.g., temporal locality phase shifts, clear hot/cold bimodality) that would benefit from zone-based or adaptive strategies?
- Could a simpler variant—e.g., counter saturation caps, different decay patterns (exponential vs. linear right-shift), or within-frequency tie-breaking refinements—improve on the parent without introducing partitioning complexity?
- How sensitive is the `capacity * 16` interval to changes in cache capacity or workload scale? Is this interval portable across different scenarios?

## Continuation

The fixed-interval frequency-decay approach from Cycle 0002 remains the strongest baseline. **Abandon zone-based partitioning entirely**: all attempts (this cycle and Cycle 0002 v002) to split the cache by access pattern hurt performance, indicating the workload benefits from unified competition on frequency alone. **Stop pursuing adaptive decay mechanisms** unless a fundamentally lower-cost signal (e.g., hit-miss ratio trending, not frequency statistics) emerges. For Cycle 0004, explore simpler **within-frequency refinements**: counter saturation behavior (when do frequencies max out?), decay patterns other than right-shift, or tie-breaking rules when evicting items of equal frequency. These avoid partitioning overhead while probing whether the current strategy has exploitable margin.
