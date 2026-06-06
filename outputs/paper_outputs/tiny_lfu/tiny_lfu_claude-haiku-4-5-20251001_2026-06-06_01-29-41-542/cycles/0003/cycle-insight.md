# Cycle 0003 Insight

## Cohort outcome

This cycle shifted focus from sketch parameter tuning (Cycles 0001–0002) to cache policy layers: eviction victim selection, counter saturation handling, and admission gate optimization. The cohort revealed that sketch tuning gains have saturated and that policy-level improvements—particularly in victim selection—are now the productive axis. Only eviction-focused changes (v002) improved hit-rate; counter recycling (v001) proved inert, and admission-gate bypassing (v003) regressed performance.

## Patterns

- **Eviction victim selection is now the active lever**: v002's frequency-recency hybrid (prioritizing age and coldness) improved hit-rate, confirming that how candidates are selected for eviction—not just admission—directly impacts performance. This marks a shift from Cycles 0001–0002, where sketch precision was the constraint.

- **Frequency signals must remain intact in decision logic**: v003's dual-tier fast-path (bypassing the sketch for hot items to reduce collision noise) degraded performance, while v002's incorporation of frequency into eviction succeeded. The pattern is clear: working around frequency uncertainty by partitioning or gates breaks admission discipline. Leveraging the existing frequency estimates more intelligently, even imprecise ones, outperforms circumventing them.

- **Counter saturation is not a practical bottleneck**: v001's adaptive recycling—dividing all sketch counters by 2 on overflow to preserve discrimination—delivered no measurable improvement. This suggests that on this workload, either saturation events are rare, or the loss of absolute frequency precision (from the global right-shift) offsets gains from continued accumulation. The 16x-width sketch is collision-tolerant enough that saturation handling has negligible impact.

- **Simple arithmetic policies outperform structural complexity**: v002's straightforward scoring formula (age / frequency + 1) was more effective than v001's targeted saturation optimization or v003's two-tier structural redesign, suggesting that interpretable, direct metrics are preferable to domain-specific tricks or circuit-breaker patterns.

## Open questions

- Can v002's frequency-recency formula be further tuned (different exponents, offsets) to extract additional gains?
- How stable is v002's improvement across varied workloads, cache sizes, or access patterns?
- Are other signals—item size, reuse distance, contention—worth combining with age and frequency in victim selection?
- Does saturation recycling matter on longer-running workloads or with different sketch initialization strategies?

## Continuation

Sketch parameter tuning is exhausted; cache policy is the new frontier. v002's success validates that eviction victim selection directly impacts hit-rate when frequency estimates are available. Pursue refinements to hybrid scoring metrics: test alternative formulations, integrate additional signals (size, cost, reuse distance), and measure across diverse workloads. Counter saturation recycling should be deprioritized unless saturation becomes common on other datasets. The admission gate's strictness remains sound and should not be revisited—the constraint is now eviction-layer quality, not admission-gate precision.
