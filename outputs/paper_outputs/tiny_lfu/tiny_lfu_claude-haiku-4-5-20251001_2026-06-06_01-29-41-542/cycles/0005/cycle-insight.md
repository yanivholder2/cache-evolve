# Cycle 0005 Insight

## Cohort outcome

This cycle attempted to refine the eviction victim-selection formula through parameter tuning and signal ablation. Three variants tested increasing the frequency offset constant, applying super-linear age weighting, and eliminating age entirely in favor of pure frequency-based selection. None of the refinements improved hit-rate; the unchanged baseline (v001) won by virtue of producing no regression.

## Patterns

- **The linear hybrid formula has reached a local optimum**: Both parameter refinements (H01's increased offset and H02's super-linear age weighting) produced zero measurable change in hit-rate. The current formula—age / (frequency + 1)—appears to be well-tuned for this workload's characteristics, and simple constant tweaking and exponent adjustments no longer yield gains. This represents a qualitative shift from Cycles 0003–0004, where policy-level changes and formula innovations still delivered improvements.

- **Recency remains essential and cannot be substituted with frequency alone**: v003's removal of age signal caused a 0.31% hit-rate loss despite the improved sketch precision from prior cycles. This definitively shows that temporal locality and frequency capture orthogonal aspects of working-set membership. The hypothesis that sufficiently precise frequency estimates might eliminate the need for age has been falsified; the two signals are complementary, not redundant.

- **Fine-tuning the frequency-recency surface exhibits diminishing returns**: Cycles 0003–0004 showed that switching from pure-LRU to hybrid scoring and then optimizing the linear formula were productive. Cycle 0005's inability to improve through constant and exponent adjustments suggests the high-level structure is sound, but further local refinement has exhausted low-hanging fruit.

- **Simplicity continues to outperform elaboration**: The unchanged v001 baseline beat both attempts to refine it, reinforcing the pattern from prior cycles that straightforward, interpretable metrics (linear hybrid scoring) are more robust than parameterized fine-tuning or structural variation.

## Open questions

- Have we exhausted the victim-selection metric itself, or are there fundamentally different scoring approaches (e.g., probabilistic, rank-based, or information-theoretic) worth exploring?
- Are other cache levers now the constraint: admission gate strictness, sketch parameter interaction, or item-level metadata (size, cost)?
- Would introducing orthogonal signals—item size, reuse distance, or inter-arrival time—unlock new gains, or are they already implicitly captured by age and frequency?
- Does performance plateau differently on other workloads, or is this plateau workload-specific?

## Continuation

The victim-selection metric has plateaued under local parameter tuning. The linear frequency-age hybrid remains sound and should be retained as the eviction policy baseline. Do not pursue further formula refinement (exponents, offsets, normalization) on this or similar workloads unless new signals are incorporated. Consider pivoting to orthogonal cache levers: test whether admission gate logic (strict > admission) or sketch parameters need re-tuning in light of this stable eviction policy, or explore whether metadata-driven signals (item size, cost, access pattern clustering) add discriminative power beyond frequency and age. The eviction-selection frontier is now either about broadening the signal set or shifting focus entirely to other parts of the cache system.
