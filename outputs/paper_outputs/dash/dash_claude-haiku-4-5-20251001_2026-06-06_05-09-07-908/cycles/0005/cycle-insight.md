# Cycle 0005 Insight

## Cohort outcome

This cycle tested three refinements: lowering frequency-gating threshold to 1 on concentrated geometry, pushing geometry allocation to its asymptote (90/10), and implementing adaptive threshold scaling based on stash bucket count. Geometry concentration finally saturated at 90/10, revealing the boundary where hit-rate losses outweigh latency gains. Frequency-gating threshold tuning on fixed geometry proved counterproductive, but adaptive threshold computation succeeded, establishing a path toward geometry-independent gating that composes robustly across allocations.

## Patterns

- **Geometry asymptote identified at ~85/15–90/10**: The four-cycle progression of sustained improvement (25/75 → 50/50 → 62.5/37.5 → 75/25 → 85/15 → 90/10) finally encounters saturation. v002's 90/10 geometry improves get and put latency but regresses hit-rate as stash capacity shrinks to 3 buckets—insufficient for the Zipfian workload's tail. This definitively closes the question: geometry concentration is not a bottomless lever. The sweet-spot appears to be around 75/25–85/15, where both metrics improve; pushing past 85/15 optimizes for latency at the expense of hit-rate.

- **Hardcoded threshold tuning is geometry-specific, not universal**: v001's attempt to lower the frequency gate from threshold=2 to threshold=1 on 75/25 geometry demonstrates that the prior cycle's selectivity (threshold=2 on concentrated geometry) was already near-optimal. Reducing it further floods the 8-bucket probing tier with false-positives, causing collision chains and hit-rate dilution. The threshold is not a knob to turn independently; it is tightly coupled to the stash tier's capacity.

- **Adaptive threshold computation unlocks cross-geometry composition**: v003's breakthrough is shifting from a hardcoded constant (`PROMOTION_THRESHOLD: u16 = 2`) to a geometry-aware initialization parameter (`max(1, stash_buckets / 4)`). This single change eliminates the sensitivity to allocation profile: on 25/75 (24 stash buckets), the threshold becomes 6; on 75/25 (8 stash), it remains 2. By scaling the gate's selectivity with the stash size, v003 achieves 42–43% get-latency improvement and 9–19% put-latency improvement while holding hit-rate stable—proving that gating can become orthogonal to geometry when parameters adapt.

- **Simple adaptive logic beats complex per-geometry tuning**: v003 demonstrates that a single geometric scaling formula outperforms cycles of manual threshold calibration across different allocations. This shift from hardcoded constants to initialization-time parameter computation is mechanically trivial but strategically powerful: it removes the human coupling between geometry and promotion logic, enabling reliable composition.

- **Hit-rate remains the harder metric to preserve**: Across all five cycles, hit-rate degradation appears whenever either geometry or gating pushes too far (e.g., 90/10 or threshold=1). Latency improvements alone (v002's get/put gains) do not compensate for hit-rate regression on Zipfian workloads, confirming the prior insight that capacity utilization is more valuable than operation-level speed on skewed access patterns.

## Open questions

- Does adaptive threshold on aggressive geometry (e.g., 85/15 or a sweet-spot near 80/20) deliver better composition than v003's baseline results, potentially reclaiming some of the hit-rate margin?
- Is there an optimal geometry between 75/25 and 90/10 (e.g., 80/20 or 82/18) that maximizes both hit-rate and latency?
- Can the adaptive scaling formula itself be tuned (e.g., `stash_buckets / 3` or `stash_buckets / 5`) to shift the trade-off frontier, or is division-by-4 near-universal?

## Continuation

Adaptive parameter scaling is the key insight: frequency-gating selectivity must track geometry allocation to remain orthogonal, and this can be achieved via a simple initialization-time computation rather than per-operation overhead. Next cycle should test adaptive threshold on aggressive geometries (85/15 or a candidate sweet-spot like 80/20) to determine whether v003's composition success generalizes and to identify whether a geometry better than 75/25 exists. The geometry asymptote is now localized to the 85/15–90/10 range; stop pushing geometry further and focus instead on finding the optimal allocation within that band, combined with adaptive gating. If adaptive thresholds on aggressive geometry maintain hit-rate while improving latency, the composition strategy is vindicated and future cycles can explore entirely different optimization axes.
