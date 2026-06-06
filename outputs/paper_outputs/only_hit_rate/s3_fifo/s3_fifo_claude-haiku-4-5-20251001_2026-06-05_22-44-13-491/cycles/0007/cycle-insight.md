# Cycle 0007 Insight

## Cohort outcome

Three refinements tested the structural-selectivity frontier: incremental reduction to 1/13, aggressive reduction to 1/14, and orthogonal compounding of 1/12 with MAX_FREQ=5. For the first time across eight cycles, the winner (v002) was not the most aggressive structural variant, but rather the pairing of proven 1/12 selectivity with deepened frequency protection. This marks the saturation boundary's arrival—the inflection point where the dominant optimization axis shifts from pure admission-gate tightening to structure-and-frequency compounding.

## Patterns

- **Saturation boundary localized to the 1/12–1/14 range**: v001's 1/13 reduction yielded only +0.0003 marginal gain, the smallest improvement magnitude across eight consecutive structural-selectivity cycles, signaling diminishing returns. v003's 1/14 improved without degradation, confirming the workload tolerates extreme selectivity but with sharply reduced impact. Together, these variants narrow saturation to a precise zone: improvements continue, but the slope flattens dramatically compared to earlier cycles (1/4→1/10 progression).

- **Bottleneck shifts from admission gate to main-queue retention at extreme selectivity**: v002's pairing of 1/12 + MAX_FREQ=5 outperformed both v001 (incremental structural, +0.0003) and v003 (aggressive structural, 1/14). This reversal from the prior seven cycles' pure-structural dominance reveals that at 1/12 selectivity, the admission gate filters so aggressively that only the highest-confidence items reach the main queue. Further tightening the gate yields minimal additional benefit; instead, extending the tenure of proven items in the main queue delivers better returns. The workload has crossed a threshold where frequency-depth protection becomes the higher-leverage secondary axis.

- **Frequency-structure compounding validates at the saturation boundary**: v002's modest but measurable gain from pairing 1/12 + MAX_FREQ=5 confirms that frequency-depth and structural selectivity compound synergistically—but only once the admission gate reaches extreme tightness. In Cycles 0005–0006, frequency-depth additions (1/8+MAX_FREQ=5, 1/10+MAX_FREQ=4) offered complementary but marginal improvements subordinate to pure structural tightening. Now, at 1/12, frequency protection becomes a legitimate primary lever. This timing shift indicates the workload's optimization landscape is discontinuous: structural selectivity is dominant until saturation, then frequency-depth control becomes competitive.

## Open questions

- **Where is the precise saturation cliff?** v003's 1/14 shows no degradation despite diminishing returns; does the true utilization collapse occur beyond 1/14, or is the boundary softer than prior patterns suggest (gradual decay rather than cliff)?

- **Can 1/13 and 1/14 unlock greater gains when paired with MAX_FREQ=5 or higher?** v002's success with 1/12+MAX_FREQ=5 raises the question: would 1/13+MAX_FREQ=5 or 1/14+MAX_FREQ=5 (or even MAX_FREQ=6) deliver better results than 1/12's pairing? The compounding effect may enable tighter structural selectivity to remain viable.

- **Is 1/12+MAX_FREQ=5 a local optimum or global?** v002 won by a narrow margin against pure-structural v003. Is the peak somewhere else in the structure×frequency interaction space—say, 1/11+MAX_FREQ=5, or 1/12+MAX_FREQ=6?

## Continuation

The structural-selectivity axis has hit saturation between 1/12 and 1/14; stop pushing pure structural reduction. The next cycle must search the structure×frequency interaction space near the saturation boundary. Test whether tighter structural selectivity (1/13, 1/14) compounds synergistically with deeper frequency protection (MAX_FREQ=5, MAX_FREQ=6), or fine-grain variants around 1/12±1 paired with MAX_FREQ=5+ to locate the true optimal point. If 1/12+MAX_FREQ=5 proves robust, pivot to workload-characterization or learned admission policies that might adapt per-phase rather than rely on fixed structural constants—this workload's transition from structure-dominant to frequency-dominant suggests it may benefit from dynamic policy selection or adaptive boundaries.
