# Cycle 0006 Insight

## Cohort outcome

The cohort revisited three previously-identified threads — a referenced-bit tie-breaker in the demotion scan, an intermediate ghost window at half of total capacity, and logical-clock frequency decay at victim-selection time — and none produced a hit-rate gain. The winner (v001) advanced on neutrality for the second consecutive cycle, confirming the policy is at a workload-a local optimum.

## Patterns

- The referenced-bit tie-breaker (v001) is structurally sound and avoids the cycle-3 position-freeze failure, but it targets a collision pattern — equal-frequency candidates where one is recently referenced — that this trace does not produce at the narrow LRU-quarter scan window. The mechanism is inert rather than harmful.
- The ghost window is now exhaustively mapped across the meaningful range: probationary capacity is too narrow to matter, half-capacity regresses, full capacity (cycle 4) regressed harder. Reuse events at distances between probationary capacity and half-total-capacity are cold traffic on this workload, not deferred-warm traffic, and the ghost mechanism cannot be rescued by window sizing alone.
- Logical-clock decay at a fixed half-life (v003) is acutely workload-sensitive. The value of the existing frequency signal comes from distinguishing demoted-from-protected entries from cold newcomers; a half-life shorter than the typical inter-access gap on this trace collapses that distinction and eliminates the scan's ability to protect warm entries — the same problem it was meant to sharpen. Any fixed decay constant is unlikely to generalize without being tuned to the workload's reuse-distance distribution.
- Two consecutive cohorts (cycles 5 and 6) have produced winners that advance on neutrality rather than gain. This is a convergence signal: the main structural gaps in the policy have been closed, and the remaining unexplored axes either require workload-specific tuning or are genuinely unproductive on workload-a.

## Open questions

- Is there any fixed-rate decay constant (e.g., half-life at 2048 or 4096 ops) that preserves the frequency distinction on workload-a, or does the inter-access distribution make any single constant wrong?
- With ghost admission, scan fractions, promotion gating, frequency caps, referenced bits, and decay all exhausted, are there structural eviction philosophies not yet tested — adaptive segment sizing, ARC-style balance feedback — that could break out of the current plateau?
- Is the policy broadly strong, or specifically tuned to workload-a? Multi-workload evaluation is now the clearest way to answer this.

## Continuation

Next cycle should treat the current policy (frequency-aware demotion and eviction, counter halving on demotion, MAX_FREQ cap, optional referenced-bit tie-breaker) as a stable baseline and pivot away from parameter tuning on workload-a. Ghost window and fixed-rate decay are both exhausted as productive directions. The highest-value move is either introducing a second workload to test transferability of the gains, or exploring a genuinely different eviction philosophy — such as adaptive segment-ratio balancing — that does not share the local optimum of the current SLRU design.
