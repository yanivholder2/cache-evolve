# Cross-Cycle Insights

## Recurring wins

- **Frequency-based eviction with uniform periodic decay is robustly optimal.** The 0.75× multiplicative decay at capacity × 16 intervals has held as the baseline from Cycle 0002 through 0009, with every subsequent perturbation either regressing or neutral. This core mechanism protects frequently-accessed items while gradually forgetting ancient hotspots, capturing the access-skewed structure of the workload.

- **LRU ordering within equal-frequency tiers provides measurable tie-breaking benefit.** Two-pass victim selection (finding minimum frequency, then head-most item within that tier) won in Cycle 0006 and remained the baseline through 0009. Even as gains diminished, it consistently outperformed or matched alternatives, indicating temporal locality has exploitable structure when frequency discrimination is held constant.

- **Fixed, simple parameters outperform adaptive and complex mechanisms.** Across Cycles 0005, 0007, 0009, every attempt at adaptive decay timing, threshold-based exemptions, per-access responsiveness, or hit-rate monitoring either failed or showed zero benefit. Fixed-interval periodic decay and uniform victim selection have correctly calibrated the workload's phase dynamics; layering adaptation on top adds overhead without addressing real transitions.

- **Decay interval capacity × 16 is robustly tuned.** Tested in multiple directions across Cycles 0004 (faster at × 8) and 0006 (slower at × 24), both regressed. The interval balances frequency accumulation against workload phase shifts with high precision; deviations destroy discriminative power.

## Recurring failures

- **Zone-based partitioning consistently degrades performance.** Tested in Cycles 0001, 0002, 0003 in multiple forms (cold/hot split, frequency-aware zones, resized partitions), all either regressed or failed. Root cause: zones create artificial promotion barriers and mask frequency signals across boundaries. Items with moderate frequency get evicted from cold zones before accumulating enough access history to earn protection. The workload requires unified frequency competition.

- **Hybrid frequency-recency scoring fails when combined with decay.** Attempted in Cycles 0007–0008 across multiple architectures and coefficients (100× weighting, 25× weighting, one-pass and two-pass selection). All regressed because periodic decay compresses frequencies into a narrow range; position-based terms then dominate and invert frequency discrimination, privileging recent accesses over genuinely hot items.

- **Selective or threshold-dependent decay breaks uniform forgetting.** Tested in Cycles 0007 and 0009 (exempting frequencies below thresholds, preserving weak signals), both regressed. Preventing some items from decaying proportionally allows weak-frequency items to remain artificially inflated, degrading eviction quality. The workload requires all frequencies to age together.

- **Per-access responsive decay contradicts frequency-based selection.** Cycle 0009 v001's activity-responsive two-tier system failed by reintroducing recency bias (items accessed within 64 ops retain full frequency). This inverts the balance the uniform periodic mechanism calibrated, prioritizing recent accesses over cumulative frequency history and recreating the failure mode of hybrid scoring.

## Open questions

- **Has structural saturation been reached?** The last four cycles (0006–0009) have produced exclusively negative or neutral results on novel approaches. After exhausting decay tuning (Cycles 0002–0005), victim-selection refinement (0006–0007), hybrid signal exploration (0008), and decay timing/precision alternatives (0009), the evidence suggests convergence to a tight local optimum. The open question is whether this represents fundamental saturation or whether a paradigm shift (e.g., non-frequency-based eviction) could unlock further gains.

- **Does the workload contain phase transitions that fixed decay already captures implicitly?** Every adaptive mechanism (Cycles 0005, 0007, 0009) failed; decay coefficient and interval refinements are locked. This suggests either the workload lacks sharp temporal behavior shifts, or periodic uniform decay fortuitously captures phase dynamics better than explicit adaptive detection. Confirming this understanding would clarify whether future exploration should accept saturation or probe new paradigms.

- **Can implementation-level micro-tuning extract marginal gains?** Counter representation precision, decay triggering efficiency, data-structure layout, or decay coefficient sensitivity between 0.74× and 0.76× remain untested. The question is whether such tuning is available and meaningful, or whether the decision boundary is so tight that granular parameter adjustment falls below measurement noise.

- **Is the workload truly frequency-dominated with negligible recency structure?** The universal failure of hybrid scoring, despite recency's theoretical appeal for temporal locality, suggests the access pattern is almost entirely governed by cumulative frequency. Confirming this through workload characterization (e.g., frequency distribution shape, reuse distance, temporal clustering) would validate the current architecture's design choices.

## Per-policy notes

- **Frequency-based eviction with uniform periodic decay (incumbent winner):** The 0.75× multiplicative decay at capacity × 16 intervals paired with two-pass LRU tie-breaking is definitively optimal for this workload. The mechanism protects hot items through cumulative frequency counting, forgets ancient hotspots through uniform periodic decay, and breaks ties within frequency tiers by recency. Deviations universally regress; this policy has reached saturation.

- **Zone-based partitioning (tested and rejected):** All attempts to split the cache by access pattern (cold/hot regions, adaptive partitions) failed because zones artificially mask frequency signals and create barriers preventing moderate-frequency items from earning protection. The workload requires unified competition; partitioning is fundamentally incompatible with the observed access distribution.

- **Hybrid and adaptive strategies (tested extensively, all rejected):** Frequency-recency hybrids fail due to signal dominance imbalance; decay compresses frequencies to narrow ranges where position-based terms invert decision boundaries. Adaptive timing and threshold exemptions fail because they add per-operation overhead and break the global synchronization of decay. The workload either lacks detectable phase shifts or benefits more from simple fixed parameters than from explicit adaptation.

- **Eviction signal design convergence:** Pure frequency discrimination is sufficient and optimal. Secondary signals (recency, temporal position, adaptive thresholds) degrade performance. Within frequency tiers, LRU provides diminishing tie-breaking benefit as decay improves discrimination. The cache has converged to a frequency-dominated eviction model; adding orthogonal signals introduces noise.
