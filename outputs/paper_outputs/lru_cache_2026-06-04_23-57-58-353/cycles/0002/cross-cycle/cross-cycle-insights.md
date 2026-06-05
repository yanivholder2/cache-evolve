# Cross-Cycle Insights

## Recurring wins

- **Two-segment SLRU** has been the most durable structural improvement: routing new entries into a probationary buffer before they can displace the protected working set consistently outperforms plain LRU and has held up across both cycles.
- **Acting at insertion time beats acting at eviction time.** Every strategy that intercepts entries before they enter the hot set (SLRU, ghost admission) outperformed strategies that only act when capacity is exceeded (CLOCK, frequency-window scan).
- **Ghost admission on top of segmentation** extended gains in cycle 2 by capturing cross-residency warmth — keys evicted from probationary that re-arrive earn fast promotion into protected, which SLRU alone does not track.

## Recurring failures

- **Frequency-aware demotion inside the protected segment** has failed in the one trial attempted (cycle 2). The root cause is a signal-design flaw: a narrow scan window covering only a fraction of the segment and counters that reset to zero on every demotion starve the mechanism of durable signal before it can discriminate high-reuse from low-reuse entries. This is not yet evidence that frequency is uninformative — only that the current implementation is too weak.
- **Widening probationary beyond the default split** hurt in cycle 2. The hot working set is large enough that shrinking the protected segment below the 80/20 baseline causes more demotion churn than the extra scan-filtering headroom saves. Ratio tuning appears to be a diminishing-return axis and is likely not worth further exploration at current workload scale.

## Open questions

- **Ghost admission contribution in isolation.** v003 (cycle 2) was built from the plain-LRU baseline, so its gain bundles SLRU segmentation and ghost admission together. Applying ghost admission on top of the established cycle-1 SLRU winner would isolate the ghost set's marginal value.
- **Frequency-aware demotion with a corrected design.** Would persistent (non-resetting) counters and a much larger scan window allow frequency demotion to recover? The idea is not exhausted — only the current implementation is ruled out.
- **Optimal ghost window size.** Ghost capacity equal to probationary capacity is the only size tried. A smaller ghost window (e.g., half of probationary) might preserve most of the benefit at lower bookkeeping cost.
- **CLOCK throughput advantage.** CLOCK eliminates linked-list pointer updates entirely. Under high concurrency or very large capacities its structural simplicity may produce measurable throughput gains even if its hit-rate ceiling is lower; this has not been tested.

## Per-policy notes

- **SLRU (two-segment):** Consistent winner across cycles 1 and 2 in multiple configurations. The 80/20 protected-to-probationary split appears to be a local optimum for the scan-heavy workload tested; looser splits degrade it. It is now the established baseline.
- **Ghost admission (LIRS-style):** Won cycle 2 as the v003 variant. Effective when traces contain recurring keys that cycle in and out of cache (re-arrival after eviction is a reliable warmth signal). Contribution not yet isolated from SLRU segmentation.
- **CLOCK:** Produced a hit-rate improvement in cycle 1 but trailed SLRU. Its eviction-time-only signal is weaker than SLRU's insertion-time partitioning. Throughput under concurrency remains an unexplored upside.
- **Frequency-window scan (eviction-time):** Bounded by a fixed look-ahead in cycle 1; frequency-aware demotion inside protected failed in cycle 2. Both failures trace to the same root: frequency signal is not durable enough when the observation window is small and counters reset. Needs a redesign before further trials.
