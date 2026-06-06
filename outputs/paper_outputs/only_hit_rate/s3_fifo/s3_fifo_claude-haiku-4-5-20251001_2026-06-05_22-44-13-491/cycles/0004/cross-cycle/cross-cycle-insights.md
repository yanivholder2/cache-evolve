# Cross-Cycle Insights

## Recurring wins

- **Small-queue admission-gate tightening (structural selectivity)**: Four consecutive cycles (0002 v003 → 0003 v001/v003 → 0004 v003) confirm that reducing the small-queue capacity ratio remains the single most effective and consistent improvement lever. Reductions from 1/4 to 1/8 all showed positive impact with no saturation boundary yet detected.

- **Orthogonal optimization of frequency-depth and admission selectivity**: Frequency-depth (MAX_FREQ) and small-queue sizing represent genuinely separate axes. When combined (0004 v002: 1/6 + MAX_FREQ=4), they deliver compounding gains, validating that both dimensions contribute real value.

- **Conservative, incremental tuning outperforms aggressive single-axis pushes**: 0001's modest 3→4 frequency increase succeeded where larger adjustments faltered; 0004's structured reduction (1/8) outperformed orthogonal compounding, suggesting the workload responds better to focused, incremental improvements on the dominant lever than scattered aggressive changes.

## Recurring failures

- **Ghost-hit frequency crediting**: Attempted in 0001 v002 (freq=2), 0002 v002 (freq=1), and implied across cycles. Consistently fails—prior survival does not predict future reuse. The baseline approach of treating ghost hits as cold (freq=0) and letting them earn frequency through actual re-access remains superior.

- **Frequency-threshold selectivity at promotion gates**: 0003 v002 (stricter freq>2 threshold for small-to-main promotion) hurt performance despite the intuition that stricter gates should filter noise. Suggests frequency signals are too coarse-grained at this decision point; size-based filtering outperforms frequency-based filtering.

- **Frequency-depth diminishing returns**: 0001→0002 shows MAX_FREQ gains fade (3→4 strong, 4→5 marginal); by 0004, no further frequency-depth testing continues, indicating the axis approaches saturation. Absolute reuse depth matters less than selectivity of item admission.

## Open questions

- **Saturation boundary for small-queue reduction**: How far does structural selectivity continue to pay off? The trend is monotonic from 1/4→1/8 with no cliff detected. Does performance degrade below 1/8 (1/9, 1/10), or can the workload tolerate even tighter admission gates?

- **Synergistic compounding at boundary**: If 1/8 is paired with MAX_FREQ=5 or beyond, do the optimizations deliver larger combined gains than either achieves independently, suggesting a high-leverage operating point?

- **Workload-informed parameter prediction**: Can reuse-depth distribution, access burstiness, or working-set size characteristics predict optimal small-queue ratio and frequency-depth a priori, enabling future cycles to target saturation directly rather than search incrementally?

- **Why size-based beats frequency-based selectivity**: The consistent pattern (frequency-threshold filters fail, size-based gates succeed) suggests frequency signals carry noise or misalignment with the workload's access pattern at the admission-gate decision. What is the underlying mismatch?

## Per-policy notes

- **S3-FIFO baseline**: All exploration has focused on two orthogonal tuning axes within S3-FIFO: small-queue admission capacity and frequency-depth (MAX_FREQ). The baseline's frequency-based eviction mechanics remain intact; tuning has been strictly parametric, not structural.

- **Frequency mechanism role**: MAX_FREQ controls how much reuse is required before items become protected. Early cycles showed this axis delivers value (0001 win), but gains saturate rapidly; frequency-depth is complementary but clearly the subordinate dimension.

- **Small-queue as the high-leverage system lever**: Four consecutive successful reductions suggest the small queue's primary role is not utilization buffering but *admission filtering*. Aggressive under-provisioning does not break working-set diversity absorption; instead, it improves hit-rate by admitting only items likely to be reused, and frequency-threshold (freq>1) effectively separates signal from noise even at tight ratios.
