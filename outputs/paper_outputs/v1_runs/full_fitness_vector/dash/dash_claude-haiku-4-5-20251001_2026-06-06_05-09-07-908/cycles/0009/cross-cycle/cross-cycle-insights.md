# Cross-Cycle Insights

## Recurring wins

- **Geometry reallocation is the dominant performance lever**: Adjusting the normal-to-stash bucket allocation ratio (from baseline 25/75 through 80/20 and beyond) has consistently improved both hit-rate and latency across cycles 0001–0006. The progression shows sustained gains until saturation near 85/15–90/10, with 80/20 emerging as the local optimum by cycle 0006 where trade-offs are minimized.

- **Adaptive parameter scaling unlocks reliable composition**: Fixed thresholds (e.g., counter ≥ 2) fail across varying geometries until cycle 0005 introduces geometry-aware formula computation. Adaptive threshold divisors that scale with stash bucket count reliably compose with geometry changes across the allocation spectrum (cycles 0005–0008), enabling orthogonal improvements without regression.

- **Hit-rate and latency improvements often coexist**: Rather than a hard trade-off, the most successful variants improve both metrics simultaneously or incur only marginal cost on one dimension. Geometry tuning and adaptive thresholding both exhibit this pattern, suggesting the workload permits alignment of capacity utilization and operation efficiency.

- **Simple mechanisms outperform complex composition when success overlaps**: When both pure geometry and geometry-plus-thresholding variants succeed, simpler approaches deliver more transparent and robust outcomes (cycles 0006–0007). Yet by cycle 0007 onwards, composition becomes reliable when parameters are inversely tuned (tighter geometry pairs with stricter gating), suggesting the framework has matured beyond parameter coupling failures.

## Recurring failures

- **Back-insert promotion is fundamentally costly**: Across cycles 0002–0004, back-insert composition fails consistently regardless of geometry size (50/50 through 85/15). Both get and put latency regress despite pairing with increasingly generous primary-tier capacity, indicating the MRU-insertion overhead is structural, not capacity-dependent. This axis is abandoned by cycle 0004.

- **Hardcoded frequency thresholds lack portability**: Cycles 0002–0004 show that a fixed counter ≥ 2 threshold works well on concentrated geometries but over-filters on balanced geometry, trapping hot items in lower tiers. Not until cycle 0005 introduces divisor-based adaptive computation does gating become geometry-independent and reliable.

- **Aggressive geometry concentration loses hit-rate at the boundary**: Cycle 0005 demonstrates that 90/10 allocation improves latency but regresses hit-rate as stash shrinks to 3 buckets—insufficient for the Zipfian workload's tail. Cycle 0009 further confirms that even small deviations from 80/20 (toward 79/21) degrade both metrics, establishing the allocation space as sharply peaked.

- **Frequency-aware eviction adds complexity without decision improvement**: Cycle 0009 explicitly tests frequency-aware item ordering in eviction logic. Implementation succeeds but provides no hit-rate advantage over pure recency (LRU), confirming frequency is not a bottleneck on this workload.

## Open questions

- **Can independent latency-optimization paths compose?** Cycle 0009 identifies two divergent successes: v002 (implementation efficiency via in-place counters) and v003 (geometry-aware formula co-optimization). Testing their composition is the critical next step—does efficiency compose with parameter tuning, or do they compete for the same latency budget?

- **Is 80/20 geometry truly locally optimal, or does a finer allocation exist?** Cycle 0009 confirms movement away from 80/20 (toward 79/21) regresses both metrics, but only single-percentage deviations are tested. A tighter search (e.g., 79/21, 78/22) might reveal nearby peaks.

- **What optimization axes remain unexplored?** Geometry, promotion logic (gating), and thresholding parameters have been heavily tuned. Eviction policy (recency vs. frequency) has been tested and dismissed. Future wins may require tier-crossing heuristics, promotion cascading policies, or workload-specific tiering strategies entirely outside current parameter space.

- **Is the adaptive divisor formula universal, or is each allocation band best served by a fixed constant?** Cycle 0008 proposes meta-parametrization (divisor computed from allocation ratio), but its success across the full spectrum (baseline through aggressive geometry) remains untested. This determines whether threshold tuning can be fully decoupled from geometry selection.

## Per-policy notes

- **LRU base with promotion filtering**: All variants build on a baseline LRU eviction policy. Promotion filters (frequency-gating with adaptive thresholds) layer on top, controlling transitions between recency-ordered tiers. Frequency-aware eviction ordering (cycle 0009) adds no value on this workload.

- **Two-tier architecture is consistent across all variants**: Normal tier (hash-indexed, hot items) and stash tier (fallback, probe-based) remain constant. Optimization focuses on the allocation ratio between them and the promotion selectivity (gating threshold) that controls flow between tiers. No alternatives (three-tier, dynamic tier sizing) have been tested.

- **Geometry and threshold tuning are inversely coupled by design**: Tighter geometry (more normal buckets, fewer stash buckets) requires stricter thresholding (higher divisor, fewer promotions) to avoid overwhelming the stash. Looser geometry permits more permissive thresholds. This inverse relationship is not a coupling to eliminate but a design principle to exploit: pair aggressive geometry with high divisors, conservative geometry with low divisors.
