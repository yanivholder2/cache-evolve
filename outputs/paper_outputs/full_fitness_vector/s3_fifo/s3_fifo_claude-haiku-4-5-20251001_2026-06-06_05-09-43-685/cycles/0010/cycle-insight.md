# Cycle 0010 Insight

## Cohort outcome

This cycle tested three refinements aimed at escaping the latency-hit-rate plateau: linear occupancy scaling to weaken re-admission protection and recover hit-rate (v001), occupancy-responsive boost combined with batch-amortized decay (v002), and frequency-threshold gating on ghost-hit re-admission to prevent main-queue crowding (v003). None achieved dual-metric wins. v001 lost both latency and hit-rate by underprotecting re-admitted items. v002 failed to preserve the cold-path overhead advantage despite careful placement. v003 succeeded in improving hit-rate (+1.14%) but at severe latency cost (36% get slowdown). The cohort reveals that **mechanism soundness does not guarantee implementation efficiency**: all three tested architecturally justified approaches that foundered on computational cost.

## Patterns

- **Occupancy-responsive boost magnitude is tightly calibrated to workload; linear scaling misses the mark**: v001's switch from quadratic (2.0 + r²) to linear (1.5 + 2.0r) scaling reduced the boost range from 2–5 to 1.5–3.5. This weakened re-admission protection, losing both the latency advantage and failing to recover hit-rate. The failure suggests the issue is not the quadratic formula's magnitude but its calibration relative to queue dynamics. Linear scaling underprotects re-access patterns in this workload, confirming that the space between linear and quadratic remains unexplored and may require non-standard functional forms.

- **Cold-path architectural advantage collapses under computational complexity**: v002 combined cycle 0009/v002's occupancy-responsive boost with cycle 0009/v003's batch-amortized decay. Despite placing occupancy computation at ghost-hit re-admission (a cold path), the floating-point arithmetic—division, squaring, min operation—introduced measurable overhead (31% get slowdown, 12% put slowdown). The "zero-cost cold-path" principle from cycle 0009 breaks when the computation itself is inherently expensive. Cold paths are only zero-cost if operations are cheap.

- **Frequency-threshold re-admission selectivity is architecturally sound but implementation-expensive**: v003's selective re-admission (only re-admitting items with demonstrated re-access patterns) delivered a clear hit-rate win (+1.14%) by preventing low-frequency ghost entries from crowding the small-queue working set. However, the implementation—extended ghost tuple structure `(K, u8)` and frequency lookup on every put—increased get latency by 36% and put by 6.8%. The mechanism is correct; the overhead is unacceptable.

- **Parameter tuning has exhausted its yield; implementation efficiency is now the bottleneck**: Cycles 0007–0009 progressed by exploring *which* signals to use and *where* to compute them. Cycle 0010 shows that refinements to those mechanisms yield only trade-offs. v001's parameter adjustment failed; v002's architectural reuse failed; v003's mechanism refinement improved one metric at severe cost to another. The frontier has shifted from signal innovation to implementation optimization.

## Open questions

- Can v003's frequency-threshold gating be optimized—e.g., by storing frequency inline in ghost entries, using bit-packed flags, or applying the check only to a subset of ghost-hit cases—to recover latency while retaining the +1.14% hit-rate improvement?

- Does the observed trade-off pattern indicate that the ghost-queue evolution has reached a Pareto frontier, or do alternative architectural approaches (e.g., adjusted small/main queue sizing ratio, different eviction order) exist that could improve both metrics simultaneously?

- For cycle 0009/v003's fixed +2 boost baseline (44% get latency, 37% put latency wins), what is the opportunity cost of the lost 1.14% hit-rate? Is the latency specialization acceptable for the workload, or must hit-rate recovery be prioritized?

- Can frequency-selective re-admission be applied selectively (e.g., only when main-queue occupancy exceeds a threshold) to reduce the hot-path overhead while preserving the crowding-prevention benefit?

## Continuation

The cohort validates that **mechanism soundness does not guarantee implementation efficiency**, and the frontier has shifted from signal design to computational optimization. V003's +1.14% hit-rate gain proves frequency-selective re-admission is architecturally correct but too expensive as implemented. V002's failure shows that floating-point occupancy calculations carry hidden costs even at cold-path placement. V001 confirms that linear scaling underprotects this workload. **For the next cycle:** (1) **if hit-rate is the priority**, focus on optimizing v003's selective re-admission mechanism—test inline frequency storage or sampled checks (every Nth ghost hit) to measure whether latency can be recovered; (2) **shelve occupancy-responsive adaptation** for now—both cycle 0009/v002 and this cycle's v002 demonstrate that adaptive boost formulas carry implementation overhead that erases theoretical cold-path advantage; (3) **validate the Pareto frontier** by testing structural alternatives (queue sizing, eviction order) rather than parameter tuning, since five cycles of refinement have yielded only trade-offs; (4) **establish a metric priority decision**: cycle 0009/v003's baseline (44% latency wins, negligible hit-rate loss) may be the production-ready solution if latency specialization is acceptable; otherwise, commit resources to optimizing v003's selective re-admission to unlock hit-rate gains without overhead cost.
