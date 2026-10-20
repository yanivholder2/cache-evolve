# Cycle 0004 Insight

## Cohort outcome

Three refinements were tested on the small-queue structural-selectivity axis: incremental reduction to 1/7, orthogonal compounding with frequency-depth (1/6 + MAX_FREQ=4), and aggressive reduction to 1/8. The winner (v003) pushed structural selectivity further, confirming that the saturation boundary for admission-gate tightening remains unlocated. Stricter small-queue filtering continues to outperform both incremental steps and attempts to compound orthogonal optimizations, indicating the workload still sits well below its utilization cliff.

## Patterns

- **Aggressive structural selectivity outperforms incremental and orthogonal approaches**: v003's jump from 1/6 to 1/8 delivered larger gains than v001's incremental 1/6→1/7 step and v002's orthogonal combination of 1/6 + MAX_FREQ=4. Four consecutive cycles (0002 v003, 0003 v001/v003, 0004 v003) confirm that small-queue reduction remains the single most effective lever; the workload rewards increasingly selective admission filtering without sign of diminishing returns.

- **Saturation boundary is further out than expected**: Both v001 (1/7) and v003 (1/8) report positive hit-rate impact with no performance cliff. The workload demonstrates robustness to aggressive under-provisioning of the small queue—items demonstrating genuine reuse still graduate successfully to main-queue storage, and frequency selectivity (freq > 1 for promotion) effectively separates signal from noise even at tighter capacities.

- **Orthogonal compounding works but remains subordinate to structural selectivity**: v002's combination of 1/6 structural selectivity + MAX_FREQ=4 improved hit-rate and validated that frequency-depth and admission-gate sizing are genuinely orthogonal axes. However, v003's larger gains from going to 1/8 confirm that structural selectivity is currently the higher-leverage dimension—frequency-depth offers complementary but smaller absolute improvements at this stage.

## Open questions

- How far can small-queue reduction continue? Does the trend persist at 1/9 or 1/10, or does 1/8 approach the saturation cliff where working-set diversity becomes over-constrained?
- If v003's 1/8 reduction is paired with MAX_FREQ=4, do the two optimizations compound to deliver even larger gains than either achieved alone?
- Can workload characteristics (reuse-depth histogram, inter-arrival burstiness) predict the optimal small-queue ratio *a priori*, allowing future cycles to target the saturation boundary directly rather than search incrementally?

## Continuation

The structural-selectivity axis remains dominant and has not yet hit saturation. Aggressively push small-queue reduction to find the utilization cliff: test 1/9 and 1/10 to locate the boundary where performance degrades. Simultaneously, test v003's 1/8 + MAX_FREQ=4 combination to confirm whether aggressive structural selectivity and frequency-depth compound synergistically at the boundary. Once saturation is mapped, shift focus to workload-adaptive or learned admission policies that might exceed fixed-ratio boundaries, or investigate whether the observed selectivity gains transfer to different workload patterns.
