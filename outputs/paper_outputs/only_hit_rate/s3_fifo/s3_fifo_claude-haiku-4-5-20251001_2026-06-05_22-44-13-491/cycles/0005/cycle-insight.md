# Cycle 0005 Insight

## Cohort outcome

Three refinements continued the structural-selectivity axis: incremental reduction to 1/9, aggressive reduction to 1/10, and orthogonal compounding of 1/8 with MAX_FREQ=5. The winner (v002) pushed small-queue reduction to 1/10 and delivered measurable improvement, extending the monotonic trend unbroken across four consecutive cycles. Notably, neither incremental reduction (v001) nor orthogonal compounding (v003) outperformed the pure structural play, confirming that tight admission-gate selectivity remains the dominant and unsaturated lever.

## Patterns

- **Structural selectivity shows no saturation sign despite five reduction steps**: The progression 1/4→1/5→1/6→1/8→1/9→1/10 has yielded consistent hit-rate improvements with no degradation or cliff. v002's aggressive 1/10 jump delivered the largest gain, and v001's incremental 1/9 step also improved despite smaller magnitude. This is a remarkable pattern: the workload continues to reward increasingly selective admission filtering at a scale (1/10 of capacity) that would be expected to trigger utilization collapse. The fact that items with genuine reuse still graduate successfully through the frequency gate suggests the workload exhibits exceptionally strong reuse locality—signal and noise are clearly separable even at extreme selectivity.

- **Diminishing returns have not yet appeared**: v001 notes "diminishing magnitude" compared to prior cycles, yet still improved. This is consistent with a gradual approach to saturation rather than an abrupt cliff. The workload appears to tolerate over-constrained small-queue sizing without failing to capture the working set, indicating the access patterns are skewed enough that frequency-based promotion (freq > 1) effectively filters repeaters even under extreme pressure.

- **Frequency-depth compounding remains complementary but subordinate**: v003's combination of 1/8 structural selectivity + MAX_FREQ=5 showed "modest but consistent improvement" and validated orthogonal compounding once more. However, v002's pure structural approach (1/10) outperformed v003, reaffirming that at this stage, tight admission-gate sizing is the higher-leverage axis. The pattern is now five cycles deep: frequency-depth and structural selectivity are orthogonal, but structural selectivity is the dominant lever by a wide margin.

## Open questions

- **Where is the true saturation boundary?** All variants this cycle improved; does the trend continue at 1/11 or 1/12, or do we find degradation soon? The absence of a cliff at 1/10 is surprising and warrants aggressive boundary search.

- **Does saturation manifest abruptly or gradually?** If saturation exists, does performance degrade sharply (utilization cliff) or gently (frequency signals become too coarse to distinguish reuse)? This affects how aggressively future cycles should search.

- **Is the extreme selectivity (1/10+) specific to this workload, or indicative of a deeper architectural mismatch?** The workload's tolerance for 1/10-ratio admission suggests either exceptional reuse clustering or that S3-FIFO's frequency mechanism is detecting signal in regimes where many cache policies would fail. Could a different admission policy (e.g., learned, adaptive, or frequency-weighted from the start) exceed fixed-ratio boundaries?

## Continuation

The structural-selectivity axis remains dominant and has not yet hit saturation—continue the aggressive boundary search. Test 1/11 and 1/12 to locate the actual utilization cliff or confirm that selectivity gains persist even further. Once saturation is mapped with confidence, investigate whether workload characteristics (reuse-depth histogram, burstiness, skew) can predict the optimal ratio *a priori*, enabling the next workload's cycles to target the saturation boundary directly rather than search incrementally. If saturation occurs beyond 1/12, pivot to adaptive or learned admission policies that might dynamically adjust selectivity per-phase rather than rely on fixed structural constants.
