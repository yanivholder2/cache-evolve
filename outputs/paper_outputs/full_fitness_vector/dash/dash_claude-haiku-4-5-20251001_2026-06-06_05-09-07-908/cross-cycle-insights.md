# Cross-Cycle Insights

## Recurring wins

- **Bucket geometry reallocation remains the primary performance lever**: Adjustment of hash-indexed vs. stash capacity ratio has driven consistent improvements across all cycles. The progression from baseline (25/75) through 50/50, 62.5/37.5, 75/25, 85/15, and into saturation (90/10) reveals geometry as the most reliable and composable axis. By cycles 0006–0007, geometry-based optimizations continue to improve both hit-rate and latency even when paired with other mechanisms.

- **Reduction of unnecessary work on the hot path**: All successful variants across cycles exploit opportunities to eliminate wasteful operations: redundant promotion attempts (cycle 0002's frequency gating), false-positive tier transitions (cycle 0005's adaptive thresholds), and over-ambitious capacity reallocation (cycle 0007's inverse-tuned divisor). Each improvement compounds by focusing work on genuinely hot keys.

- **Adaptive parameter scaling enables reliable composition**: The breakthrough from cycle 0005 onward—scaling promotion thresholds to stash bucket capacity via divisor-based formulas—solved the brittleness that plagued earlier compositions. By cycle 0007, inverse-tuned parameters (aggressive geometry paired with higher threshold divisor) achieve simultaneous improvements across both hit-rate and latency, confirming that composition works reliably when parameters align with workload capacity constraints.

- **Hit-rate and latency improvements coexist**: Rather than trading one metric for the other, successful late-cycle variants (0006–0007) preserve or improve both simultaneously. This convergence indicates they exploit the same underlying efficiency: capacity utilization and hot-key residence time remain the binding constraints on Zipfian workloads.

## Recurring failures

- **Back-insert promotion is fundamentally costly**: Showed initial promise in cycle 0001 but failed consistently from cycle 0002 onward. Incurs latency overhead on both get and put operations across all tested geometries (25/75 through 85/15). The MRU-insertion mechanism carries structural overhead incompatible with this workload; no amount of primary-tier capacity (cycles 0004, 0006) recovers it.

- **Hardcoded promotion thresholds lack geometry awareness**: Cycles 0002–0006 demonstrate that fixed frequency-gating constants (e.g., counter ≥ 2) are geometry-specific: threshold=2 over-filters balanced geometry (cycle 0003) but under-filters concentrated geometry (cycle 0004). Only adaptive, divisor-based formulas that scale with stash bucket count enable reliable composition across allocations.

- **Geometry extremes violate workload requirements**: Pushing allocation to 90/10 (cycle 0005) sacrifices hit-rate for latency; stash shrinks below functional threshold (3 buckets insufficient for Zipfian tail). Optimal lies in the 75/25–80/20 band where both metrics improve without trade-off.

- **Aggressive parameter combinations without inverse-tuning cause interference**: Cycle 0006's v001 paired tight geometry (85/15) with lenient threshold tuning (divisor 4), amplifying promotion pressure and triggering eviction cascades. Composition only succeeds when parameters are inversely scaled: tighter geometry requires stricter filtering (higher divisor).

## Open questions

- **Pareto frontier among cycle 0007 winners**: Do the three successful variants (85/15 + divisor 2, 80/20 + divisor 3, baseline + divisor 2) represent a strict ordering or incomparable trade-offs across hit-rate and latency percentiles? Is one dominant, or should future cycles explore all three?

- **Universal divisor meta-formula**: Can a single formula (scaling divisor by both geometry ratio and stash bucket count) achieve reliable composition across the entire allocation spectrum, or is each geometry band best served by a fixed constant?

- **Permissiveness frontier**: Cycle 0007 uses divisor 2 (most permissive) on baseline geometry. Does further reduction (e.g., 1.5, 1, or geometry-adaptive permissiveness) yield additional gains without regressing hit-rate, or has the frontier been reached?

- **Orthogonal optimization axes beyond promotion and geometry**: All cycles focus on promotion ordering, bucket allocation, and frequency thresholds. Eviction policies (beyond LRU), bucket sizing, tier-crossing heuristics, or adaptive workload-dependent tuning could unlock improvements orthogonal to the current parameter space.

## Per-policy notes

- **Single two-tier LRU framework across all cycles**: All variants operate within a hash-indexed tier and stash tier with frequency-gated promotion between them. No alternative eviction policies (LFU, LIRS, ARC, etc.) were tested; all optimizations refine this single policy's parameter space.

- **Zipfian workload consistently guides optimization**: All successful improvements prioritize capacity allocation (geometry tuning) and hot-key identification (frequency gating) over operation-level acceleration, confirming that workload's bottleneck is capacity utilization, not per-operation latency. Improvements align with the skewed nature of the trace.

- **Adaptive initialization-time parameter computation as universal enabler**: The shift from hardcoded constants to divisor-based formulas that scale with stash capacity (cycles 0005–0007) eliminated the coupling between specific geometry instances and policy tuning. This single mechanism—geometry-aware threshold initialization—enables reliable composition and should become the default pattern for future cycles.
