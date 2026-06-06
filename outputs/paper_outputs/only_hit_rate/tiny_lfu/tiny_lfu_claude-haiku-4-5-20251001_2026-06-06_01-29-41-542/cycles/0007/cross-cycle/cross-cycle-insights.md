# Cross-Cycle Insights

## Recurring wins

- **Frequency-recency hybrid for eviction**: Cycles 0003–0004 established that a linear formula (age / (frequency + constant)) beats pure-LRU and single-signal alternatives. This has been the only repeated winner across multiple cycles.
- **Measurement precision over logic rewrites**: Sketch width improvements (0001) and sketch depth tuning (0002) beat attempts to change admission thresholds or frequency-decay logic. Refine the signal itself rather than work around it.
- **Simplicity and directness persist**: Unchanged baselines, straightforward formulas, and linear scoring consistently outperform complex refinements (tiering, normalization, collision heuristics, time-based gates).

## Recurring failures

- **Admission-gate relaxation**: Gates tested in cycles 0001 (>= vs >), 0003 (skip for hot items), 0006 (>= vs >) all regressed. The strict `>` is not an artifact of earlier policies; it remains optimal under refined eviction as well.
- **Complex signal combinations**: Normalization by max-frequency, burstiness tracking, frequency tiering, size penalties, and time-based admission all failed or regressed. These additions introduced noise rather than refined discrimination.
- **Fine-tuning after structural convergence**: Once the linear frequency-age formula was established (cycles 0003–0004), all attempts to refine via constants (0005), new signals (0006–0007), or alternative methods (0007) yielded no gain. Parameter-space returns are exhausted after structure is found.

## Open questions

- **Theoretical limits**: What is the asymptotic hit-rate ceiling for this workload under the current policy, and how far below it is the incumbent?
- **Generalization**: Does the frequency-age hybrid eviction and strict admission remain optimal across other workloads, cache sizes, or threshold values, or are findings workload-specific?
- **Alternative frequency estimation**: Could sampling-based, probabilistic, or adaptive frequency-tracking schemes outperform Count-Min sketches?
- **Architectural alternatives**: Does a fundamentally different eviction structure (adaptive policy mixing, multi-level caches, cost-aware replacement) exist beyond the current policy framework?

## Per-policy notes

- **Admission gate**: Strict `>` (not >=) threshold has proven robust across six cycles and multiple attack vectors (relaxation, skip logic, time-based variants). Not a limitation of earlier eviction policies; remains optimal under refined policies as well.
- **Eviction policy**: Evolved from pure-LRU → linear frequency-age hybrid (cycles 0003–0004, structure optimal). Resistant to local improvement: parameter tuning (0005), new signals (0006–0007), and alternative methods (0007) all failed. Policy appears locked to a stable optimum.
- **Sketch parameters**: Width and depth tuning (0001–0002) delivered measurable but diminishing returns; counter saturation recycling had no impact (0003). Collision heuristics failed (0007). Sketch is dimensioned adequately; not the active improvement lever.
- **Frequency signal**: Must be used directly—linear division in victim scoring outperforms sqrt-damping, normalization, or tiering. The signal is robust; do not "correct" for perceived noise or bucket into tiers.
