# Cross-Cycle Insights

## Recurring wins

- **Structural selectivity (small-queue admission-gate tightening) dominates the early-to-mid optimization landscape**: Seven consecutive cycles (0002–0007) confirmed that aggressive reduction of small-queue capacity (1/4 → 1/5 → 1/6 → 1/8 → 1/9 → 1/10 → 1/12) yields consistent hit-rate improvements. Pure admission-gate tightening outperformed all alternatives until saturation arrived in Cycle 0007, establishing it as the single highest-leverage optimization axis across the explored range.

- **Conservative frequency-depth scaling (MAX_FREQ 3→4) establishes baseline orthogonal gains**: Cycle 0001's single-level increase from MAX_FREQ=3 to MAX_FREQ=4 delivered measurable improvement and was validated in later cycles as a complementary but subordinate optimization axis. Frequency-depth protection compounds synergistically with structural selectivity but does not replace it as the primary lever.

- **Saturation boundary can be precisely localized**: By Cycle 0007, the inflection point where structural selectivity hits saturation and frequency-depth becomes the competitive secondary lever was narrowed to the 1/12–1/14 range. At 1/12 + MAX_FREQ=5, the pairing outperformed both incremental structural reduction (1/13) and aggressive structural reduction (1/14) alone, marking the first win for structure×frequency compounding and signaling the transition from pure structural dominance to interaction-space optimization.

## Recurring failures

- **Ghost-hit frequency crediting consistently fails**: Across Cycles 0001, 0002, and 0003, variants attempting to boost frequency credits on re-admission (freq=2 upfront, modest earned protection, or stricter promotion thresholds) either degraded performance or showed no measurable improvement. The pattern is unambiguous: prior cache presence does not predict future reuse, and frequency gains must be earned through actual re-access, not gifted through admission policies. This approach should be retired entirely.

- **Frequency-based selectivity at decision gates yields no benefit**: Cycle 0003's v002 (raising promotion frequency threshold from freq>1 to freq>2) failed to improve hit-rate despite the intuition that stricter frequency filtering would improve selectivity. Size-based admission-gate reduction outperforms frequency-threshold tuning at every tested decision point, suggesting frequency signals are too coarse-grained or misaligned with workload reuse patterns at the small-queue promotion stage.

- **Frequency-depth scaling remains subordinate until structural saturation**: Cycles 0005–0006 tested frequency-depth compounding (1/8+MAX_FREQ=5, 1/10+MAX_FREQ=4) and showed only modest improvements, consistently outperformed by pure structural tightening. The pattern held until Cycle 0007, when extreme structural selectivity (1/12) enabled frequency-depth to become a legitimate co-primary lever. This indicates the optimization landscape is discontinuous: frequency-depth additions deliver marginal gains in the pre-saturation regime and must await structural tightening to unlock full compounding value.

## Open questions

- **Where is the precise utilization cliff for structural selectivity?** Variants tested 1/13 and 1/14 with diminishing returns but no degradation in Cycle 0007. Does the true utilization collapse occur beyond 1/14, or is the boundary softer than historical cliff patterns suggest (gradual decay rather than sharp failure)? Future cycles should aggressively probe this boundary.

- **Do tighter structural selectivity and deeper frequency protection compound synergistically at saturation?** Cycle 0007's v002 (1/12+MAX_FREQ=5) won by narrow margin, raising the question: would 1/13+MAX_FREQ=5, 1/14+MAX_FREQ=5, or even 1/12+MAX_FREQ=6 deliver better results? The compounding effect may enable the optimization landscape to shift further along the structure-frequency frontier.

- **Can workload characteristics predict optimal structure-frequency pairs a priori?** The workload's remarkable tolerance for extreme selectivity (1/10, 1/12) while maintaining graceful frequency-based filtering suggests exceptional reuse clustering or burstiness. Can reuse-depth histograms, inter-arrival distributions, or access-pattern skew metrics predict the optimal (small-queue ratio, MAX_FREQ) pair for new workloads, allowing cycles to target the saturation boundary directly rather than search incrementally?

- **Would adaptive or learned admission policies exceed fixed structural constants?** The workload's transition from structure-dominant (Cycles 0002–0006) to frequency-dominant (Cycle 0007) suggests benefit from dynamic policy selection. Could per-phase admission selectivity or frequency-protection adjustment, informed by observed access patterns, outperform fixed (small-queue ratio, MAX_FREQ) tuples?

## Per-policy notes

- **S3-FIFO family**: All cycles tested S3-FIFO or S3-FIFO variants. The core findings relate to tuning the small-queue admission gate (capacity ratio) and the main-queue frequency-eviction threshold (MAX_FREQ). No alternative cache policies were evaluated, so observed patterns are specific to S3-FIFO's frequency-based eviction mechanics and two-queue architecture. Key insight: S3-FIFO's frequency thresholds are finely calibrated, and the small queue acts as a gating mechanism where admission selectivity is the dominant lever until extreme tightness (1/12) enables frequency-depth protection to become competitive.
