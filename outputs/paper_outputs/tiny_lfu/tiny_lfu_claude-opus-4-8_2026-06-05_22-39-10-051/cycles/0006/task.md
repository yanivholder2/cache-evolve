# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0006

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0006/hypotheses/H03.md

## Prior cycles digest

### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants forked from cycle 0002's wide-sketch winner and stayed on
the estimator-fidelity axis: another 4x sketch-width step (v001), conservative-
update increments (v002), and an isolated admission-only doorkeeper with a long
clear window (v003). v001 and v002 landed in the Winners, but both only matched
the parent within noise; the doorkeeper (v003) regressed and is refuted.

## Patterns

- The estimator-fidelity lever has flattened. Once the count-min was widened in
  cycle 0002, every further accuracy move — more width, or conservative update —
  bought essentially nothing, because cells already separate the trace's
  distinct keys. Width and CU turned out to be redundant with each other, not
  additive, on this workload.
- Both "Winners" this cycle won by *not regressing*, not by lifting hit-rate.
  The real signal is that the accuracy axis has been exhausted: there are no
  more collisions left to remove.
- Extra admission filtering is the wrong direction, again. The bare
  `incoming_freq > victim_freq` test already screens one-hit-wonders, so even a
  carefully isolated long-window doorkeeper only delayed warming keys and cost
  hit-rate — consistent with cycle 0002's doorkeeper loss.

## Open questions

- With both the policy-design and estimator-accuracy axes now mined out, is
  there any remaining lever on this trace, or is hit-rate at this capacity
  effectively capped?
- Would conservative update finally pay off only if paired with a deliberately
  *narrower* sketch (trading the now-wasted width memory for contention CU can
  correct) — i.e. is there a width/CU combination cheaper than the parent at
  equal accuracy?

## Continuation

Stop sweeping estimator fidelity: width has plateaued and conservative update
adds nothing on top of the wide sketch — both are now parked as exhausted. The
admission-doorkeeper line is fully refuted across two cycles; drop it for good.
Next cycle should change axis entirely rather than refine accuracy: probe
whether a narrower-sketch + conservative-update tradeoff can hold accuracy at
lower memory, or treat this workload's hit-rate as near its ceiling and shift
the search toward efficiency/cost dimensions instead of raw hit-rate.


### Cycle 0004

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


### Cycle 0005

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
