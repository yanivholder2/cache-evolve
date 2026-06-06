# Cross-Cycle Insights

## Recurring wins

- **Simplicity and interpretability outperform parameterized elaboration**: Direct metrics (linear age/(freq+1), frequency-based partitioning) consistently beat sub-linear damping, normalization, tiering, and collision heuristics. When a straightforward formula or method works, refinements reliably fail.
- **Hierarchical deterministic partitioning breaks global-scoring plateaus**: Moving from all-at-once continuous scoring to two-pass ordered partitioning (frequency-cold cohort with fallback) unlocked gains after five consecutive failed tuning cycles. This shift revealed the prior architecture's stability was not fundamental optimality but method-dependent saturation.
- **Frequency and age remain orthogonal and complementary**: Removing age causes measurable regression; frequency-based primary partitioning outperforms age-based, yet layering age as a secondary signal preserves its value. Neither signal is redundant.
- **Strict admission gate (>) is architecturally essential**: The > threshold has survived every relaxation attempt (>=), every policy shift, and every eviction refinement. Its selectivity is not a limitation of prior layers but a fundamental discriminator that tied frequencies must reject.

## Recurring failures

- **Gate relaxation consistently regresses**: Switching > to >= across Cycles 0001 and 0006 (and retested implicitly in 0003) fails regardless of eviction sophistication. The selectivity is not an artifact of simpler victim selection.
- **Fine-tuning locally optimal architectures converges without gain**: Parameter adjustment attempts (Cycle 0005: offset and exponent tweaks; Cycle 0009: threshold percentile shifts) in stable architectures show diminishing returns collapse to zero. Once a system reaches local maximum under an architecture, constant and exponent tuning becomes ineffective.
- **Augmenting locked policies with new signals fails uniformly**: Size penalties, time-dependent admission logic, burstiness tracking, and sketch collision heuristics (Cycles 0006–0007) all failed or regressed when added to the established frequency-age framework. New signals either conflict with frequency's discrimination or lack proper grounding.
- **Sketch parameter tuning saturated early**: Width scaling (4x→8x→16x) delivered rapidly diminishing returns by Cycle 0002. Adaptive counter recycling and collision-detection heuristics yielded no measurable improvement, signaling the sketch is no longer the active bottleneck.

## Open questions

- **Can frequency and age partitioning be layered hierarchically?**: Would frequency-cold cohort, then age-ranked selection within that subset, compound both signals' discriminative power without reducing victim-pool availability?
- **Does the 25th-percentile frequency threshold generalize?**: Is it specific to the tiny-lfu trace and configuration, or robust across workloads, cache sizes, and admission-gate settings?
- **What is the hit-rate ceiling under hierarchical partitioning?**: Is the asymptotic limit near, or is substantial room remaining above the current optimum?
- **Do secondary partitioning signals unlock gains?**: Can burstiness, inter-arrival variance, or reuse-distance clustering improve discrimination as second-pass criteria within the cold cohort?

## Per-policy notes

- **Sketch parameters**: Width and depth improvements exhausted by Cycle 0002 with rapidly diminishing returns; subsequent saturation-handling and heuristic refinements (Cycles 0003, 0007) produced no improvement. The sketch is no longer the optimization axis.
- **Admission gate**: Strict > threshold is stable across all policy changes. Relaxations (>=), time-based variants, and bypass mechanisms have all failed. Gate selectivity is architecturally necessary and non-negotiable.
- **Eviction—global scoring era (Cycles 0003–0007)**: Linear age/(freq+1) hybrid emerged as locally optimal under all-at-once scoring. Parameter and signal augmentation attempts regressed or stalled, creating the appearance of fundamental optimality.
- **Eviction—hierarchical partitioning era (Cycles 0008–0009)**: Frequency-cohort partitioning (25th-percentile cold threshold, age-ranked fallback) broke the global-scoring plateau and improved hit-rate. The threshold calibration is locally tight; adjacent percentiles regress. Frequency is the stronger primary filter; age remains valuable as a secondary discriminator.
