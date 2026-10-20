# Cross-Cycle Insights

## Recurring wins

- **Bucket geometry concentration is the most reliable and consistent performance lever.** Across all eight cycles, sustained improvement from baseline 25/75 allocation through 75/25–80/20 range demonstrates geometry is not a solved problem. The progression shows no attenuation until saturation boundary around 85/15–90/10, where hit-rate begins to regress.

- **Adaptive parameter scaling (threshold divisor based on stash capacity) enables reliable composition.** Introduced in cycle 0005, refined through cycle 0008, this approach decouples promotion logic from geometry by making selectivity inversely track allocation tightness. All three variants in cycle 0007–0008 succeed simultaneously using fixed geometry + varying divisor, confirming this is the matured approach to composition.

- **Simple, pure-geometry variants consistently outperform complex compositions when both strategies win.** Across cycles 0006–0008, whenever a geometry-only variant and a geometry-plus-gating variant both succeed, the simpler mechanism delivers cleaner outcomes (no surprising trade-offs, no latency regressions). Simplicity appears to be a robust signal for optimization stability on this workload.

- **Reducing unnecessary hot-path operations improves both metrics simultaneously.** From cycle 0001's insight that eliminating redundant LRU climbs and false-positive promotions pays off in hit-rate and latency, the pattern holds: capacity-preserving optimizations (fewer eviction cascades) and operation-level efficiency (fewer traversals) reinforce rather than trade against each other on Zipfian workloads.

## Recurring failures

- **Back-insert MRU-promotion carries structural overhead that geometry gains cannot absorb.** Tested across cycles 0001, 0002, 0004 on increasingly aggressive geometries (50/50 through 85/15), back-insert consistently regresses latency on both get and put operations without compensating hit-rate gains. The overhead is fundamental to the mechanism, not capacity-dependent.

- **Hardcoded threshold constants do not generalize across geometries without regressing hit-rate.** Cycles 0003, 0005, 0006 demonstrate that a fixed threshold (e.g., counter ≥ 2) is too conservative on balanced geometry and too lenient on concentrated geometry, causing either false-positive filtering or cascading over-promotion. This failure was resolved only by adaptive formula that scales threshold with stash bucket count.

- **Composition without parameter alignment triggers cascading regressions.** Cycles 0003, 0006 show that logically orthogonal improvements (geometry and gating) fail when parameters push in the same direction (e.g., aggressive geometry + aggressive threshold). The lesson: composition requires inverse tuning (tighter threshold on tighter geometry), and this coupling cannot be ignored through mechanical independence.

- **Pushing geometry concentration beyond the saturation zone (>85/15) trades hit-rate for latency gains.** Cycle 0005 identifies the boundary: 90/10 improves get and put latency but loses hit-rate as stash shrinks to 3 buckets, insufficient for the workload's tail access pattern. The frontier is around 75/25–80/20, not further.

## Open questions

- **Does the adaptive divisor formula (scaling based on geometry's normal-to-stash ratio) succeed across all allocation ratios, or only within the 75/25–85/15 band?** Cycle 0008's H02 proposal is the next frontier; if universal, it eliminates the need for future threshold tuning. If allocation-specific, further investigation is required.

- **Is 80/20 the true optimal geometry, or do finer-grain allocations (78/22, 82/18) exist within the saturation zone?** Cycle 0006 established 80/20 as the clean midpoint; cycles 0007–0008 did not exclude it from frontiers. Narrow search within the 75/25–85/15 band could identify a tighter optimum.

- **What optimization axes remain unexplored beyond geometry allocation and threshold-based gating?** Eight cycles of refinement within two dimensions (bucket split and promotion selectivity) suggests saturation of these levers. Candidates: eviction policies beyond LRU, tier-crossing heuristics, bucket-size tuning, or workload-adaptive strategies.

- **What is the precise Pareto frontier among cycle 0007–0008's three winning variants?** Each uses different geometry + divisor combinations and succeeds on different metrics. Do they represent a true frontier with unavoidable trade-offs, or does one dominate the others?

## Per-policy notes

- **LRU eviction policy remains constant across all cycles.** No variants tested alternative eviction strategies (LFU, frequency-biased, adaptive recency-vs-frequency). All optimizations focus on tier management and promotion logic within the LRU framework, leaving the core eviction mechanism unexamined as a potential future lever.

- **Promotion frequency thresholding within LRU tiers is the primary tuning axis.** Cycle 0005's shift to adaptive threshold computation represents the mature approach: instead of per-tier or per-operation logic, a single initialization-time parameter (divisor) controls selectivity across all operations and scales automatically with allocation.
