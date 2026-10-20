# Cycle 0008 Insight

## Cohort outcome

Three refinements tested the structure–frequency interaction space at the saturation boundary: incremental structural reduction to 1/13 paired with MAX_FREQ=5, extended frequency depth to MAX_FREQ=6 at proven 1/12 selectivity, and extreme structural selectivity at 1/14 paired with MAX_FREQ=5. The winner (v003) confirmed that aggressive admission filtering remains viable at the saturation cliff when paired with sufficient frequency protection, reversing the prior cycle's apparent saturation signal.

## Patterns

- **Extreme structural selectivity saturates only when frequency protection is insufficient**: Cycle 0007 showed 1/14 alone yielding marginal returns, suggesting structural saturation. Cycle 0008's v003 (1/14 + MAX_FREQ=5) reverses this conclusion: the apparent saturation was not a structural ceiling but rather a mismatch between aggressive admission filtering and shallow main-queue retention. Items passing the 1/14 gate are high-utility, and extending their tenure unlocks the potential that tight selectivity created. The workload's reuse clustering is strong enough to sustain both extreme admission filtering and extended item lifetime simultaneously.

- **Frequency-depth protection is the higher-leverage axis at the saturation boundary**: v002's extension from MAX_FREQ=5 to MAX_FREQ=6 yielded a measurable +0.0014 gain, substantially larger than v001's incremental structural step (+0.0002). This reversal mirrors Cycle 0007's inflection point, where compounding became the dominant axis. However, Cycle 0008 reveals the asymmetry: within the saturation zone, deepening frequency protection scales more favorably than further tightening the admission gate. The gains remain positive but diminishing in both dimensions—yet frequency depth still has unlocated leverage.

- **Structure and frequency form a coherent, discontinuous optimization surface**: Nine cycles of evidence now reveal a non-monotonic landscape: admission-gate selectivity dominates until saturation (Cycles 0002–0006), frequency protection becomes competitive at the boundary (Cycles 0007–0008), and their pairing compounds synergistically—but *only* if the admission gate is tight enough. v003's success over v002 and v001 indicates the peak lies beyond 1/12 selectivity, contingent on sufficient frequency depth. This suggests workload-aware policy tuning requires joint optimization, not sequential refinement of isolated dimensions.

## Open questions

- **What is the true limit of the structure–frequency compounding effect?** v003 wins with 1/14 + MAX_FREQ=5, but can even tighter structural selectivity (1/15, 1/16) sustain performance if paired with deeper frequency (MAX_FREQ=6, MAX_FREQ=7)? The gains remain positive at both extremes; the boundary may lie significantly further out.

- **Is there a peak in the structure×frequency interaction space, or does compounding scale indefinitely?** v002's gain (+0.0014) and v001's (+0.0002) suggest gains are still available, but can we characterize the trade-off surface to predict optimal pairing? Finer-grain search (1/12–1/14 × MAX_FREQ 5–7) may reveal whether there is a local optimum or a monotonic frontier.

- **Can reuse-depth histograms or clustering metrics predict the optimal structure–frequency pairing *a priori*?** This workload has now demonstrated that extreme selectivity (1/14) is viable with appropriate frequency protection. Do access-pattern characteristics (skew, burstiness, reuse clustering) enable prediction of the saturation boundary and optimal pairing for a new workload without iterative search?

## Continuation

The structure–frequency interaction space remains unsaturated and productive. Stop searching the margins of single dimensions; instead, systematically explore the joint structure×frequency space near and beyond the current winner (1/14 + MAX_FREQ=5). Test whether even tighter structural selectivity (1/15, 1/16) compounds with elevated frequency depth (MAX_FREQ=6, MAX_FREQ=7), or whether a more finely resolved grid around the current winner (1/13–1/14 × MAX_FREQ=5–6) reveals a sharper optimum. Once the true saturation boundary is confidently mapped in the two-dimensional space, investigate whether workload introspection (reuse-depth histogram, skew metric, burstiness) can predict the optimal point *a priori*, or pivot toward learned or adaptive policies that dynamically adjust selectivity and frequency based on observed access patterns.
