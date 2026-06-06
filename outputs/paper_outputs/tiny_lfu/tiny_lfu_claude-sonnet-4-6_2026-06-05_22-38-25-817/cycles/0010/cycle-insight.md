# Cycle 0010 Insight

## Cohort outcome

All three variants tested the remaining policy knobs called out by cycle 0009: further PROMO_THRESHOLD reduction, frequency-ordered protected demotion, and shrinking the protected segment to widen probationary. All three regressed; v002 (frequency-scan demotion) was selected as winner on the basis of the smallest regression, marking two cycles in a row with no net advancement.

## Patterns

- **PROMO_THRESHOLD has a hard floor at 6.** The pattern of unlocking thin Zipfian frequency slices that succeeded from 8→6 does not continue to 5; the frequency-5 cohort is sparse enough that the additional protected-segment churn it generates outweighs any admission benefit. This axis is conclusively closed.
- **Frequency-scan demotion from protected is unproductive regardless of the admission threshold.** Tried first under PROMO_THRESHOLD=8 (cycle 0009 v003) and now under PROMO_THRESHOLD=6, the scan finds no meaningfully colder candidate than the LRU head. Protected's LRU ordering already approximates frequency ordering sufficiently; the knob has no room left.
- **The SLRU split is exhausted in both directions.** Enlarging protected was probed and closed in cycles 0002–0005; shrinking protected (widening probationary by ~50%) now confirms the opposite direction is also unprofitable. The k=4 probationary victim scan already provides adequate shelter for warming items at the 10% probationary size, so the extra buffer buys nothing while the cost to hot residents is immediate.
- **Every structural policy axis identified across this run is now closed.** Sketch quality (closed 0006–0008), PROMO_THRESHOLD (floor reached 0010), victim-scan widths, demotion order, and SLRU split ratio are all exhausted.

## Open questions

- Is the remaining hit-rate gap attributable entirely to irreducible workload entropy — cold-start misses on the Zipfian long tail that no in-cache eviction policy can capture?
- Would a structurally different admission gate (e.g., a Bloom-filter-style cold-item filter at the probationary entry point) address any portion of the remaining gap, or does the gap belong to the workload rather than the policy?

## Continuation

The design has reached its ceiling for this workload and trace: all known tuning axes are closed and the last two cycles have produced no advancement. The next cycle's hypotheses-writer should either declare convergence and halt, or pivot to genuinely architectural changes (a pre-admission filter, an alternative base eviction policy) rather than re-probing closed knobs. Incremental tuning of the existing TinyLFU structure is unlikely to yield further gains.
