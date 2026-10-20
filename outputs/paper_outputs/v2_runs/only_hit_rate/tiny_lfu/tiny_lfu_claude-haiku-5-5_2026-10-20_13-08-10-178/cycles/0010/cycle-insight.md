# Cycle 0010 Insight

## Cohort outcome

The cohort tried three changes: widening the probation victim scan from three to four entries, scoping counter aging to the saturated row, and removing sketch increments on reads. The only Winner, v002, widened the scan; its hit-rate gain is very small. The aging change had no measurable effect, and removing read-side increments hurt.

## Patterns

- The victim-selection family produced the cohort's only gain for the second cycle running. The step from three to four candidates helped by less than the step from two to three, which suggests the benefit is tapering rather than compounding.
- Per-row aging is untested, not neutral. Its saturation threshold is so high that the halving branch almost certainly never fires on this trace, so the result says nothing about scoped versus table-wide aging.
- Read-side sketch increments carry real popularity signal. Dropping them left frequently read residents looking colder than they are, and they were evicted in favor of newcomers.
- Every result comes from one trace at one cache size, so the scan-width gain is provisional.

## Open questions

- Is the scan-width gain a real plateau past four entries, or noise at this trace and size?
- Does per-row aging help once the threshold is low enough that saturation actually occurs?
- Would a write-only sketch update work on a trace where reads and writes are less correlated?

## Continuation

Next cycle, confirm the victim-scan width on a second workload or cache size before building further on it, and check whether five candidates still pays off or the gain has plateaued. Drop read-side increment removal in its current form. Do not judge per-row aging yet: rerun it with a threshold low enough to fire on this trace before deciding whether it belongs in the cache.
