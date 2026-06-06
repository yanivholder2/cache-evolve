# Cross-Cycle Insights

## Recurring wins

- **Frequency-based protection is the consistent winner.** From Cycle 0001 through 0008, pure frequency discrimination—without zone partitioning, temporal mixing, or adaptive complexity—has outperformed all alternatives. The winning approach has converged to: 0.75× multiplicative decay at capacity × 16 intervals, coupled with two-pass LRU tie-breaking within equal-frequency tiers.

- **Fixed, simple tuning beats adaptive complexity.** Cycles 0003, 0005, and 0008 all demonstrated that once a mechanism is tuned, adding adaptive logic (frequency-distribution-aware decay, exponential approximation, hybrid scoring) degrades performance. The workload either lacks the structural properties (sharp phase shifts, multimodal access patterns) that justify adaptive overhead, or the cost of tracking adaptation signals outweighs phase-shift benefits.

- **Unified competition outperforms partitioning.** Both zone-based strategies (Cycles 0001–0003) and selective threshold-dependent decay (Cycle 0007 v002) failed by introducing artificial barriers. Allowing all items to compete fairly on frequency discrimination—without exemptions or promotion thresholds—remains superior.

- **Decay interval (capacity × 16) is a robust anchor.** Tested in both directions—acceleration (capacity × 8, Cycle 0004) and deceleration (capacity × 24, Cycle 0006)—the original interval consistently outperformed alternatives. The workload's phase shifts are gradual; deviations disrupt the balance between frequency accumulation and timely forgetting.

- **Victim-selection precision within equal-frequency tiers has margin.** Cycle 0006's two-pass LRU tie-breaking demonstrated that once decay is tuned, orthogonal refinements in eviction precision still yield gains. This remained the last measurable improvement before saturation.

## Recurring failures

- **Zone-based partitioning (cold/hot splits, resizable zones).** Tested in Cycles 0001, 0002, and 0003 under various configurations, all regressed or underperformed. The fundamental flaw: artificial promotion barriers mask frequency signals that matter across the entire cache. Items with moderate frequency get evicted from cold zones before demonstrating enough reuse to earn hot-zone protection; unified competition is superior.

- **Threshold-dependent and selective decay.** Cycle 0007 v002 (preserving weak frequencies < 4) and Cycle 0003 v003 (zone-dependent decay rates) both regressed. Shielding newly-accessed items from decay breaks frequency discrimination by allowing ephemeral patterns to retain inflated weights. Uniform proportional decay across all frequencies is required.

- **Mixing recency or temporal signals into frequency-based eviction.** Cycle 0001 v003 (ghost-list second-chance) failed through overly conservative placement. Cycle 0008 v001–v003 (hybrid frequency-recency scoring) uniformly regressed across distinct architectures (one-pass, two-pass) and parameter choices (100×, 25× weighting). The decay mechanism's frequency discrimination is so tightly tuned that adding positional or temporal noise overrides its gains.

- **Accelerating decay frequency.** Cycle 0004 v002 (capacity × 8 interval) actively hurt hit-rate by forcing the cache to forget historical patterns too aggressively. The workload requires slower, more careful forgetting; aggressive resets destroy discriminative power before new patterns stabilize.

- **Parameter sensitivity around local optimum.** Cycle 0005's failure of coefficient adjustments (0.8×, 0.7×, exponential approximation) and Cycle 0008's failure of coefficient reduction (25× vs. 100×) demonstrate that small deviations from tuned values regress. Once converged, the optimization window has closed at that parameter level.

## Open questions

- **Has the cache reached saturation for this workload?** After 8 cycles and exhaustion of orthogonal levers (decay tuning, victim-selection precision, hybrid scoring), every variant that deviates from the incumbent regresses. Is this a true local optimum, or are there unexplored orthogonal mechanisms (e.g., counter quantization, decay triggering efficiency, entirely different eviction paradigms)?

- **Why does pure frequency so completely dominate over frequency-recency hybrids?** The 0.75× multiplicative decay compresses frequencies into a narrow range, but the failure of hybrid scoring was uniform and independent of coefficient tuning. Is the workload's frequency distribution sufficiently Zipfian or multimodal that temporal mixing always introduces noise?

- **Can the decay coefficient and interval be further micro-tuned?** The 0.75× and capacity × 16 values are locked in, but Cycle 0005–0006 only tested coarser-grained alternatives. Could fine-grained exploration (0.74×, 0.76× or capacity × 15, 17) extract marginal gains, or is the plateau truly at the tested values?

- **Are there phase shifts or mode transitions in the workload that remain unexploited?** Cycles 0002–0003 explored adaptive decay; the failure of adaptive mechanisms was attributed to overhead, not workload structure. Does the access pattern genuinely lack sharp transitions, or could a lower-cost adaptation signal (e.g., hit-miss trending, not frequency statistics) unlock further gains?

- **Should exploration pivot toward implementation-level optimizations or entirely different eviction paradigms?** Cycle 0004 v001 tested counter saturation (neutral impact). Should future work focus on decay triggering efficiency, data-structure optimizations, or acceptance that frequency-based caching has reached its local optimum on this workload?

## Per-policy notes

- **Frequency-based eviction (0.75× decay, capacity × 16 interval, two-pass LRU selection).** This is the only policy sustained across all 8 cycles and the foundation of the current incumbent. It evolved from simple global counters (Cycle 0001) through periodic decay refinement (Cycle 0002), multiplicative decay tuning (Cycle 0004), and victim-selection precision (Cycle 0006). The policy is locked in as conclusively optimized; no further changes to decay mechanics or tie-breaking rules are planned.

- **Zone-based partitioning (Cycle 0001–0003, abandoned).** Initially tested as a way to protect hot items via separate LRU zones or cold/hot tiers. Refined in Cycle 0002 with hybrid frequency+LRU logic but regressed in Cycle 0003 under all variants. Core failure: artificial partition boundaries create promotion barriers that mask frequency signals. Unified competition on a single frequency axis is superior.

- **Ghost-list / second-chance (Cycle 0001, abandoned).** Attempted to track evicted items and grant re-insertion on re-access. Failed due to conservative middle-position placement that prevented effective protection of valuable items. Frequency-based approaches rendered this mechanism obsolete; never revisited.

- **Hybrid frequency-recency scoring (Cycle 0007–0008, definitively abandoned).** Tested as an alternative eviction signal combining frequency (×100 or ×25 weighting) and recency (position_from_head). When paired with the decay mechanism, all variants regressed uniformly, independent of selection architecture or coefficient tuning. Conclusion: decay's frequency discrimination is so tightly calibrated that mixing temporal bias introduces noise that overrides benefits. This approach is conclusively ruled out.

- **Decay mechanism tuning pathway (Cycle 0002–0008, locked in).** Decay emerged as the primary lever for improvement. Explored periodic resets (Cycle 0002), multiplicative decay gentleness (Cycle 0004, settled on 0.75×), coefficient sensitivity (Cycle 0005, bounded by 0.8× and 0.7× failures), interval validation (Cycle 0006, confirmed capacity × 16), and threshold-dependent variants (Cycle 0007, regressed). The mechanism is locked in and conclusively tuned; exploration should not revisit decay unless entirely new decay paradigms are proposed.

- **Victim selection within equal-frequency tiers (Cycle 0006–0008, locked in).** Two-pass LRU tie-breaking (find minimum frequency once, then select head-most victim with that frequency) was the last orthogonal improvement identified and remains stable as the best eviction-time decision rule. Hybrid scoring attempts to refine this mechanism (Cycle 0008) failed; the two-pass approach is locked in.
