# Cross-Cycle Insights

## Recurring wins

- **Simplicity and interpretability outperform complexity**: From sketch tuning through policy refinement, straightforward metrics (linear hybrid, direct arithmetic) consistently beat elaborated schemes (sqrt-dampening, normalization, recursive optimization). The linear victim-scoring formula age/(frequency+1) and the strict admission gate both exemplify transparent, robust decisions that resist improvement through parameter tuning.

- **Frequency and recency signals are orthogonal and complementary**: Demonstrated consistently from Cycle 0003 through 0006. Frequency alone cannot capture temporal locality; age alone misses working-set structure. Removing age entirely (Cycle 0005 v003) caused a 0.31% regression despite improved frequency precision, proving the two signals address distinct working-set dimensions. Hybrid scoring that weights both has remained the productive axis.

- **Eviction victim selection is the active policy lever**: Once sketch precision was refined (Cycles 0001–0002), shifting focus to victim-selection scoring (Cycle 0003 onward) consistently delivered improvements. This became the primary tuning axis after sketch parameters saturated.

- **Measurement precision matters only when it's the bottleneck**: Early gains from sketch width (4→8: +0.0047) diminished sharply with further scaling (8→16: +0.0006), and parameter refinement on stable metrics (Cycle 0005) yielded zero improvement. Precision tuning follows the constraint—once the current mechanism is refined, further refinement on the same dimension produces flat returns.

## Recurring failures

- **Relaxing selectivity constraints isolates backfires**: Gate relaxation from > to >= failed multiple times (Cycle 0001 v003, Cycle 0006 v001), confirming that strict admission is not a limitation of weaker eviction but a necessary discipline. Tied frequency estimates should evict, not admit. The gate's selectivity cannot be compensated downstream.

- **Mechanisms that decay, discard, or bypass frequency signal degrade performance**: Frequency decay (Cycle 0001 v002) halved counters too aggressively; dual-tier fast-path (Cycle 0003 v003) tried to skip collision noise but sacrificed signal entirely. Early-earned frequency is genuine and stable; working around measurement uncertainty by partitioning or gating performs worse than leveraging the noisy estimates directly.

- **Orthogonal signals without proper grounding fail to combine**: Size penalties (Cycle 0006 v002) conflicted with frequency, causing hot small items to be evicted in favor of cold large ones. Time-based admission (Cycle 0006 v003) lacked grounding—absolute elapsed time grows unbounded regardless of item value. Adding dimensions requires structural decoupling or subordination, not seamless folding.

- **Counter saturation is not a limiting factor**: Adaptive recycling (Cycle 0003 v001) produced no measurable gain, suggesting saturation is rare on this workload or the loss of absolute precision from right-shifting offsets gains from continued accumulation. The 16x-width sketch is collision-tolerant enough that saturation handling remains inert.

## Open questions

- **Is the current plateau local or global?** Cycles 0005–0006 show victim selection has locked into a local optimum and gate strictness is non-negotiable. Does this reflect a ceiling under this workload, or are different signals (cost, access clustering, burstiness) or fundamentally different policies (probabilistic scoring, rank-based selection) capable of breaking through?

- **Can admission discrimination improve without relaxing the gate?** All gate modifications failed. Are there non-frequency signals (request pattern, inter-arrival statistics, cost-per-byte) that could sharpen admission decisions while maintaining strict selectivity?

- **Should size be addressed orthogonally rather than integrated?** Size and frequency operate on conflicting optimization axes. If byte-efficiency matters, is size better addressed via separate mechanisms (size-aware clustering, cost-aware replacement) or subordinate gates, rather than folded into victim scoring?

- **Are there domains beyond sketch parameters and victim scoring that remain unexplored?** Admission logic is locked; eviction victim selection is stable. Are candidate selection pools, hash seeding strategies, or item-level metadata (reuse distance, contention) worth investigating?

## Per-policy notes

- **Sketch-based frequency estimation**: Count-Min width proved productive (4x→8x: +0.0047), while depth added modest gains orthogonally. Saturation handling remained inert, and counter precision beyond 16x width showed diminishing returns. The sketch is now collision-tolerant enough for the workload; further sketch tuning is unlikely to yield improvements.

- **Frequency-recency hybrid victim selection**: The linear formula age/(frequency+1) is stable and remarkably hard to improve. Sqrt-dampening (Cycle 0004 v002) and normalization (v003) both failed; parameter adjustments (Cycle 0005) yielded zero gain. Recency cannot be eliminated without a 0.31% regression. This metric appears to be locked into a local (possibly global) optimum for the current workload.

- **Admission gate logic**: The strict > threshold is non-negotiable across both relaxation attempts (Cycle 0001, Cycle 0006). Gate strictness is not an artifact of weak eviction downstream; it's fundamental to selectivity and survives the maturation of eviction policy. Time-based and non-frequency signals do not ground admission effectively.
