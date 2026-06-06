# Cycle 0006 Insight

## Cohort outcome

Three refinements continued the structural-selectivity search: incremental reduction to 1/11, orthogonal compounding of 1/10 with MAX_FREQ=4, and aggressive reduction to 1/12. The winner (v003) extended the monotonic improvement trend unbroken across seven consecutive cycles, confirming that the admission-gate saturation boundary remains unexplored and that aggressive selectivity continues to outperform both incremental steps and frequency-depth compounding.

## Patterns

- **Aggressive structural selectivity decisively outperforms incremental and orthogonal variants**: v003's jump from 1/10 to 1/12 delivered the largest gain, surpassing v001's incremental 1/11 step and v002's frequency-depth compounding. This reinforces the consistent pattern from Cycles 0002–0005: pure structural tightening of the admission gate is the single highest-leverage optimization axis. The workload exhibits remarkable tolerance for extreme selectivity—items with genuine reuse still graduate to main-queue storage through the freq > 1 threshold even at 1/12 capacity, indicating the access patterns are skewed enough that frequency-based filtering remains effective under severe under-provisioning.

- **No saturation cliff appears even at 1/12**: The progression 1/4→1/5→1/6→1/8→1/9→1/10→1/11→1/12 has now yielded seven consecutive improvements with zero degradation. The fact that v003's aggressive jump outperformed v001's incremental step suggests the saturation boundary may lie significantly tighter than 1/12. The workload continues to reward selectivity-at-scale without showing sign of utilization collapse, pointing to either exceptional reuse clustering or that frequency-based filtering can separate signal from noise in regimes where many cache policies would fail.

- **Frequency-depth remains complementary but subordinate even at the 1/10 frontier**: v002's MAX_FREQ=4 addition to the proven 1/10 selectivity delivered only modest improvement (+0.0009), reaffirming that structural selectivity is the dominant lever by a wide margin. Eight cycles of evidence now show that frequency-depth and admission-gate sizing are orthogonal but unequally weighted; at current selectivity levels, extending item lifetime in the main queue offers marginal gains compared to stricter admission filtering.

## Open questions

- **Can aggressive reduction continue beyond 1/12?** Does the trend persist at 1/13 or 1/14, or have we finally approached the saturation cliff? The absence of degradation at 1/12 is consistent with prior cycles and warrants further aggressive probing.

- **Does 1/12 pair synergistically with frequency-depth scaling?** Cycle 0005 tested 1/8+MAX_FREQ=5 but 1/12+MAX_FREQ=4 has not been tested. Could tighter structural selectivity unlock greater compounding with frequency-depth at an even more constrained boundary?

- **Is this extreme selectivity workload-specific, or indicative of S3-FIFO's broader potential?** The tolerance for 1/12-ratio admission is exceptional and warrants investigation: could reuse-depth histograms or burstiness metrics predict whether such extreme selectivity is optimal *a priori*, or does this workload represent an outlier in access-pattern skew?

## Continuation

The structural-selectivity axis remains dominant and unambiguously unsaturated—continue the aggressive boundary search without hesitation. Test 1/13 and 1/14 to rapidly locate the actual utilization cliff or confirm that gains persist even further. Once saturation is confidently mapped, investigate whether combining the tightest viable structural selectivity with higher MAX_FREQ values (e.g., 1/12+MAX_FREQ=4 or 1/12+MAX_FREQ=5) can unlock synergistic compounding at the boundary. If structural gains plateau, pivot toward adaptive or learned admission policies that might dynamically adjust selectivity per-phase rather than rely on fixed structural ratios—this workload's pattern suggests it may benefit from policies that respond to observed reuse-depth distributions in real-time.
