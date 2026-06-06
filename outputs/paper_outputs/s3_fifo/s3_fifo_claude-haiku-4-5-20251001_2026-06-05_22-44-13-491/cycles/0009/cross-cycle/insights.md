# Cross-Cycle Insights

## Recurring wins

- **Structural selectivity (small-queue admission-gate tightening)** remains the single most consistent improvement lever across eight consecutive cycles (0002–0009), delivering uninterrupted hit-rate gains as the ratio tightened from 1/4 to 1/15. No degradation or utilization cliff appeared until extreme ratios, indicating the workload exhibits robust reuse clustering that tolerates severe under-provisioning of the small queue.

- **Structure and frequency compound synergistically at the saturation boundary.** Once admission-gate selectivity becomes extreme (1/12+), frequency-depth protection (MAX_FREQ scaling) transitions from a complementary lever to a necessary multiplier. Cycles 0007–0009 show winners pairing tight structural ratios with elevated MAX_FREQ, reversing the prior dominance of pure structural tightening and confirming the optimization surface is discontinuous, not a smooth trade-off.

- **Aggressive optimization in one dimension does not compensate for loose calibration in the other.** Cycle 0009's decisive loss of 1/12+MAX_FREQ=7 (moderate structure with extreme frequency) demonstrates the surface is sharply peaked; items that never clear the tight admission gate cannot benefit from extended retention, making structural selectivity the foundational axis.

## Recurring failures

- **Ghost-hit frequency crediting consistently fails across all variants tested (Cycles 0001–0003).** Every approach—granting elevated initial frequency on re-admission, modest earned protection, stricter promotion thresholds—lost to baselines treating ghost hits as cold (freq=0). The pattern suggests prior cache survival does not predict future reuse in this workload; items must earn frequency protection through actual access in the live working set.

- **Frequency-based selectivity at the promotion gate (small-to-main graduation) never outperforms size-based admission filtering.** Cycle 0003 v002's attempt to tighten the promotion frequency threshold lost despite the same intuition (stricter selectivity) succeeding with size-based capacity constraints, indicating frequency signals are too coarse-grained or misaligned at this decision point compared to structural gating.

- **Incremental steps within a saturating dimension yield diminishing returns that cannot compete with dimensional shifts.** Cycle 0009 v001's +0.0007 gain at 1/15 (incremental structural step) is dwarfed by prior cycle jumps and loses to v002's frequency deepening, confirming the diminishing-return zone has arrived; further structural tightening alone will not sustain winning improvements.

## Open questions

- **Where lies the true optimum in the structure×frequency interaction space?** Current evidence points to the 1/14–1/15 selectivity region with MAX_FREQ=5–6, but precise pairing and boundary behavior (can 1/16+ sustain gains? does MAX_FREQ=7 unlock new regimes?) remain unmapped. Fine-grained sweep of 1/14–1/16 × MAX_FREQ=5–7 would locate the peak.

- **Can workload characteristics predict optimal structure–frequency pairing *a priori*?** This workload's exceptional tolerance for extreme selectivity (1/14–1/15) differs markedly from typical cache behaviors. Do reuse-depth histograms, access-pattern skew metrics, or burstiness measures predict when extreme structure is optimal, enabling future workloads to target the saturation boundary directly rather than via incremental search?

- **Why does the workload so severely punish moderate structural selectivity paired with deep frequency?** Cycle 0009 v003's decisive loss suggests items below the 1/14 admission threshold are predominantly noise; does the access-pattern signature (clustering, temporal distribution) uniquely characterize this regime, and could introspection enable avoiding such mismatches?

- **Is saturation manifesting as a sharp peak or as a gradual plateau?** Cycle 0009 shows the smallest recent gains (+0.0007, +0.0003), consistent with flattening rather than cliff. Understanding whether the boundary is abrupt or soft affects how aggressively future cycles should probe beyond the current frontier.

## Per-policy notes

- **S3-FIFO:** Frequency-based eviction is finely calibrated and exhibits a discontinuous optimization landscape. Admission-gate capacity (small-queue ratio) dominates across Cycles 0002–0006, then frequency-depth protection becomes competitive at saturation (Cycles 0007–0009). The policy's frequency mechanism cannot distinguish earned vs. gifted protection; treating ghost hits as cold and requiring re-access in the live set aligns better with this workload's access dynamics than any heuristic frequency grant. The extreme selectivity ultimately viable (1/15+) suggests S3-FIFO's frequency-based promotion threshold (freq > 1 for graduation) effectively separates signal from noise even under severe under-provisioning.
