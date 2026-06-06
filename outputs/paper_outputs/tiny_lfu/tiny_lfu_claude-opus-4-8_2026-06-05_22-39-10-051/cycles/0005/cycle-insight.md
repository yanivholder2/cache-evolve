# Cycle 0005 Insight

## Cohort outcome

All three variants forked from cycle 0004's SLRU winner and refined the
split rather than changing axes: frequency-ranked victim sampling inside
the probationary tier (v001), a frequency contest gating protected-tier
promotion (v002), and shrinking the protected cap to two-thirds (v003).
v002 took the Winner slot on a hair-thin lift; v001 also nudged just past
the parent, while v003 regressed and is refuted.

## Patterns

- Spending the sharp sketch on *which key leaves or stays within the SLRU
  segments* is directionally right in both forms tested — victim choice
  (v001) and promotion gating (v002) — but each clears the noise floor by
  only a hair. The shared cause is unchanged from prior cycles: the
  admission gate plus accurate sketch already keep the hot set resident, so
  these refinements only fire on the rare case where the LRU head hides a
  key much colder, or much hotter, than recency order implies.
- The protected-fraction tuning question from cycle 0004 is now answered:
  the SLRU favours a *larger* protected tier, not a smaller one. Enlarging
  probation traded away protection without buying admission accuracy, so
  the marginal SLRU effect is a workload ceiling, not a downward-tuning
  miss.
- Recency order within both the protected and probationary segments already
  correlates closely with sketch frequency on this trace, which is why the
  frequency contests so rarely reverse a promotion or eviction.

## Open questions

- With victim choice, promotion gating, and protected sizing all explored
  and each marginal, is there any structural lever left on this trace that
  clears noise, or is hit-rate at this capacity now conclusively capped?
- Do the two confirmed-positive refinements (v001 sampling, v002 promotion
  gate) compound if combined, or do they overlap on the same rare cases and
  net to nothing?

## Continuation

Treat the SLRU structural axis as essentially exhausted for hit-rate:
victim sampling and frequency-gated promotion are confirmed real but
marginal, and the protected fraction is best left large — the smaller-
fraction line is refuted, do not revisit. The accumulated evidence across
cycles 0003–0005 strongly indicates this workload's hit-rate is at its
ceiling. Next cycle should either test whether the two positive refinements
compound (a cheap, final hit-rate probe) or decisively pivot the search to
efficiency/cost dimensions rather than chasing further hit-rate.
