# Cross-Cycle Insights

## Recurring wins

- **Simple, transparent metrics beat domain-specific tricks**: Across Cycles 0003–0004, straightforward formulas (age / (frequency + 1)) outperformed sqrt-dampening, relative-frequency normalization, and counter recycling heuristics. Cycle 0001's success with pure sketch-width widening (no logic changes) over decay-and-relax variants confirms this pattern. Interpretable, arithmetic scoring survives workload variation better than specialized optimizations.

- **Frequency precision is a high-ROI bottleneck**: Cycles 0001–0002 show that improving frequency-estimation fidelity (via sketch width and depth) directly translates to hit-rate gains. The sketch is the signal source for both admission and eviction; measurement noise is the constraint, not the policy logic itself.

- **Eviction victim selection is the productive policy layer**: Cycles 0003–0004 both delivered improvements through eviction-side refinements (hybrid scoring), after sketch parameter tuning saturated. The eviction layer is now the active lever for optimization—where frequency estimates are consumed in practice.

## Recurring failures

- **Structural workarounds for noisy signals don't work**: Cycle 0003's dual-tier fast-path (bypassing the sketch for hot items to reduce collision noise) regressed performance, as did Cycle 0004's relative-frequency normalization. The pattern: attempting to partition decisions or filter around signal noise breaks downstream discipline. Improving signal quality itself (wider sketch, refined metrics) is more robust than designing around imprecision.

- **Weakening gate logic to tolerate measurement uncertainty**: Cycle 0001's relaxation from strict `>` to `>=` allowing tied-frequency evictions hurt hit-rate. The gate's strictness is not the problem; it correctly guards hot residents when frequency estimates collide. Loosening the gate to accommodate sketch noise is the wrong direction—the noise should be fixed upstream.

- **Saturation-handling as an active tuning target**: Cycle 0003's adaptive counter recycling (halving on overflow) delivered no measurable improvement, suggesting either saturation is rare on this workload or the loss of absolute precision from division offsets gains from continued accumulation. Counter saturation is not a practical bottleneck when sketch width is sufficient.

## Open questions

- Can the Cycle 0004 winning formula (age / (frequency + 1)) be further tuned with exponents or offsets to extract additional gains, or has the linear relationship saturated?

- How stable is the linear frequency-recency hybrid across varied workloads, cache sizes, and access patterns? Does it generalize or only fit this specific test cohort?

- Beyond frequency and age, what other signals—item size, reuse distance, contention, insertion order—would meaningfully refine victim selection without introducing complexity that undermines robustness?

- Is the sketch width-to-depth balance optimal, or do they saturate independently? Should sketch depth be re-explored now that eviction policy is the active layer?

- Would alternative frequency-estimation structures (HyperLogLog, lossy counting, exponential decay with short windows) escape the Count-Min parameter-tuning plateau, or is the sketch family itself sufficient given refined eviction logic?

## Per-policy notes

- **Admission gate (strict >)**: Remains sound and performant. Do not revisit logic. The gate's role is to preserve hot residents; it is not too strict. Further optimization should focus on improving the frequency estimates it consumes, not on relaxing its selectivity.

- **Sketch parameters (width, depth, counters)**: Width and depth tuning delivered early gains but entered diminishing returns by Cycle 0002. Width scaling (4→8→16x) shows sub-linear payoff; depth is productive but modest. Counter saturation is not limiting. Further tuning of Count-Min parameters likely yields negligible gains; consider whether a structurally different approach is needed if sketch limits are hit.

- **Eviction victim selection**: Now the primary tuning axis after sketch saturation. Simple frequency-recency hybrids (age / (frequency + 1)) consistently improve and outperform both complex normalizations and sqrt-dampening. This layer is sensitive to formula details and is the frontier for continued refinement.

- **Counter recycling and adaptive strategies**: Not a bottleneck on observed workloads. Avoid further investment in saturation-handling heuristics unless saturation becomes endemic on longer-running or higher-cardinality datasets.
