# Cross-Cycle Insights

## Recurring wins

- **Geometry concentration toward hash-indexed capacity**: Sustained improvement across all three cycles (baseline 25/75 → 50/50 → 62.5/37.5 → 75/25). No saturation observed; each concentration step improved both hit-rate and latency. Confirms hash-indexed buckets consistently outperform stash tier capacity on this Zipfian workload.

- **Eliminating unnecessary work on the hot path**: All successful variants reduce redundant operations—back-insert avoids LRU climbs; geometry rebalancing avoids stash traversals; promotion gating prevents cascading false-positive demotions. These wins span across cycles and suggest the baseline left significant margin for efficiency improvements.

- **Exploiting Zipfian skew through capacity reallocation**: Every winning variant captures the non-uniform trace distribution. Concentration favors resident hot keys; gating protects them from eviction by cold false-positives; both amplify benefit to the dominant workload subset.

## Recurring failures

- **Assuming orthogonality guarantees clean composition**: Cycle 0002 showed back-insert + 50/50 geometry regressed latency despite mechanical independence. Cycle 0003 repeated this pattern: frequency gating worked cleanly on 62.5/37.5 geometry but hurt hit-rate on 50/50. Composition depends critically on parameter values, not just axis independence.

- **Over-conservative threshold tuning across geometry regimes**: The frequency-gating threshold (counter ≥ 2) was well-calibrated for concentrated geometry (62.5/37.5, 75/25) but filtered too many legitimate promotions on balanced geometry (50/50). Threshold selectivity must match the underlying allocation ratio's candidate density.

- **Saturation hypothesis on geometry progression**: Cycle 0002 speculated that 62.5/37.5 might be approaching asymptote. Cycle 0003 disproved this; 75/25 delivered sustained gains without regression, signaling further room up the concentration axis.

## Open questions

- **Geometry asymptote**: Does 75/25 represent saturation, or does further concentration (85/15, 90/10) continue to improve both latency and hit-rate?

- **Adaptive frequency threshold**: Should the promotion gate's threshold scale with geometry allocation (e.g., threshold = max(2, ⌊stash_buckets / scaling_factor⌋)) to maintain optimal selectivity across different capacity ratios and enable clean composition?

- **Composition on highly concentrated geometry**: Does frequency gating compose cleanly with 75/25 (or more aggressive) geometry, or will the gate's overhead exceed its false-positive filtering benefit?

- **Performance on non-Zipfian workloads**: All wins thus far exploit Zipfian skew. How do geometry concentration and frequency gating degrade on uniform or bimodal distributions?

- **Granular promotion filtering**: Could per-bucket frequency counters, or different gating strategies per tier, unlock further refinement beyond global threshold tuning?

## Per-policy notes

- **Eviction policy**: All cycles evaluate a two-tier promotion-based policy with target/probing stash and hash-indexed capacity.

- **Capacity rebalance as primary lever**: Tier geometry consistently dominated performance across all three cycles. The promotion logic (ordering, gating) fine-tunes but does not supersede capacity allocation in impact magnitude.

- **Cache churn sensitivity**: The policy exhibits sharp sensitivity to false-positive promotions, which trigger evictions of genuinely hot items. Frequency gating's success stems from breaking this churn cycle, not from filtering cold items. This suggests the policy's eviction selection, rather than promotion rules, is the bottleneck.

- **Composition non-linearity**: The policy's interactions between promotion logic and geometry allocation are parameter-dependent. Simple strategies (geometry-only variants) often outperform complex compositions on this policy, indicating optimization margin comes from capacity allocation more than logic tuning.
