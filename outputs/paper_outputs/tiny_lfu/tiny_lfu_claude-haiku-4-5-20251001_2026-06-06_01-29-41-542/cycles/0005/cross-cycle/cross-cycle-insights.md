# Cross-Cycle Insights

## Recurring wins

- **Precision over permissiveness**: Improvements that refine measurement accuracy (wider sketch, hybrid metrics) consistently outperform relaxations of thresholds or logic (gate threshold relaxation, frequency decay). Measurement fidelity is more valuable than flexibility.

- **Frequency-recency hybrids are complementary signals**: Pure-frequency and pure-recency (LRU) approaches both lose to balanced combinations. This pattern held from the first policy-level test (Cycle 0003) through subsequent refinements; the two signals capture orthogonal aspects of working-set membership.

- **Simplicity and directness outlast complexity**: Straightforward arithmetic formulas (age / (frequency + 1)) beat normalization, structural workarounds, and domain-specific tricks. Interpretable metrics remain more robust as the system is refined.

## Recurring failures

- **Workarounds for uncertain measurements fail**: Attempts to circumvent or bypass frequency estimation (gate relaxation in Cycles 0001, 0003; dual-tier fast path in Cycle 0003) all degrade performance. Working intelligently with imprecise estimates outperforms designing around them.

- **Frequency accumulation is better than refresh cycles**: Periodic decay (Cycle 0001's v002) and counter recycling (Cycle 0003's v001) either degraded or stalled performance. Monotonic accumulation preserves signal; artificial resets destroy it.

- **Parameter fine-tuning of optimized formulas yields stalling returns**: Cycle 0005 showed that tweaking constants and exponents on an already-good formula (age / (frequency + 1)) produced zero improvement. Once a formula is locally optimal for its signal set, adjustment-based refinement exhausts quickly.

## Open questions

- **Have eviction metrics saturated, or are orthogonal signals necessary?** Cycles 0003–0005 extracted gains from hybrid frequency-recency scoring, then plateaued under parameter tuning. Do size, cost, reuse distance, or contention signals unlock further gains, or is the frequency-age surface already maximal?

- **Is the admission gate truly stable, or does it need re-tuning with refined eviction policy?** The gate (strict > admission) was frozen after Cycle 0001. With sketch precision and eviction policy now substantially improved, does the gate's assumptions remain sound across workloads, or does improved eviction ordering change the optimal gate threshold?

- **What governs the frequency distribution shape?** Cycle 0004's finding that linear weighting beats sub-linear damping suggests the workload's frequency distribution is steep (few very-hot items). Does this hold across workloads, cache sizes, or access patterns?

- **Is the plateau workload-specific or architectural?** Do other datasets and cache configurations show similar saturation points, or is the current plateau dependent on this workload's characteristics?

## Per-policy notes

- **Sketch-based frequency estimation**: Count-Min width scaling (4→8→16) and depth (→6) delivered gains for two cycles, then entered diminishing returns. The 16x-width sketch is collision-tolerant enough that counter saturation handling adds no practical value. This surface is largely exhausted unless fundamental alternatives (e.g., streaming frequency structures beyond Count-Min) are explored.

- **Admission gate (strict > logic)**: Remains sound and untuned since Cycle 0001. Every attempt to relax, bypass, or augment it regressed performance. The gate's selectivity is well-matched to the workload, though its behavior under orthogonal changes to sketch precision or eviction policy is unexplored.

- **Eviction victim selection**: The active policy lever across Cycles 0003–0005. Frequency-recency hybrid formulas (age / (frequency + 1)) consistently beat pure-LRU baselines and simpler alternatives. The formula appears locally optimal; further gains likely require new signals or a shift in how candidates are ranked.
