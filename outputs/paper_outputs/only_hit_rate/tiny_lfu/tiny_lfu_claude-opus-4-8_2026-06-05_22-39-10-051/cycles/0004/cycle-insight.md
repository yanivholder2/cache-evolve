# Cycle 0004 Insight

## Cohort outcome

This cycle finally changed axes off estimator fidelity: it directly
combined width×CU (v001), layered frequency-ranked victim sampling onto the
accurate sketch (v002), and split the main space into a protected/probationary
SLRU (v003). v003 took the Winner slot, but only on a hair-thin lift — every
new lever this cycle proved real-but-marginal rather than transformative.

## Patterns

- The accuracy axis is now closed by *direct* test, not inference: building
  width and conservative-update together landed exactly where each landed
  alone, confirming the cycle-0003 redundancy claim that had only been
  inferred from separate variants.
- The two structural levers — victim sampling and SLRU recency-protection —
  are each independent of estimator accuracy and each directionally positive,
  yet both pay almost nothing here. The shared cause is the admission gate:
  the `incoming_freq > victim_freq` test plus a sharp sketch already keep the
  hot working set resident, so there is little left for either mechanism to
  rescue.
- Recurring signature of this workload: a mechanism only matters when the LRU
  head hides a key much colder, or much hotter, than the gate already detects.
  That situation is rare on this trace, which is why every structural win
  hovers at the noise floor.

## Open questions

- Would victim sampling or SLRU pay off materially only under a larger sample,
  a smaller protected fraction, or a more skewed/churnier workload — i.e. is
  the marginality a tuning issue or a workload ceiling?
- With accuracy closed and two structural levers each nearly inert, is there
  any mechanism left on *this* trace that clears noise, or is hit-rate at this
  capacity effectively capped?

## Continuation

Treat the estimator-accuracy axis as definitively closed — width×CU
redundancy is now proven, not inferred; do not revisit. The structural levers
(SAMPLE victim scan, SLRU) are confirmed independent and directionally correct
but marginal against the current admission gate; keep them only as the live
substrate and probe whether parameter tuning (larger sample, smaller protected
fraction) or a churnier workload unlocks them — do not expect lift from them
unchanged. Strongly consider that this workload's hit-rate is near its ceiling
and shift the search toward efficiency/cost dimensions rather than chasing
further hit-rate.
