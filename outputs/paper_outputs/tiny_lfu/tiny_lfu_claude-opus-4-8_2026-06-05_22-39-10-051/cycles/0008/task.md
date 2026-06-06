# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0008

## Winners

- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0008/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

Forked from cycle 0005's promotion-gate winner, all three variants stayed
on the SLRU axis and probed where else the sketch signal could steer
movement: stacking victim sampling onto the promotion gate (v001), applying
sampling to the protected-tier *exit* / demotion choice (v002), and adding a
floor threshold on protected entry (v003). v002 took the Winner slot on a
hair-thin lift; v001 added nothing over its parent, and v003 was an
inadvertent no-op.

## Patterns

- The compounding bet from cycle 0005 is now answered and refuted: victim
  sampling and frequency-gated promotion do *not* stack. Both mechanisms fire
  on the same scarce "head hides a hotter key" events, so layering them
  overlaps rather than adds — confirming they target one shared rare case,
  not disjoint ones.
- The one new placement for the sketch signal — the protected-tier *exit*
  (which node to demote on overflow) — is directionally right and edged
  out the parent, but only because it occasionally spares a frequency-hot,
  recency-stale resident. Its room to move is tiny: protected recency order
  already tracks sketch frequency closely, the same workload signature seen
  across cycles 0003–0005.
- A "lever" that never actually engages produces exactly parent results. The
  entry-floor variant was inert not because the idea is wrong but because
  increment ordering made the gate unsatisfiable — a reminder that null
  results need a mechanism check before they count as evidence about the
  workload.

## Open questions

- Every distinct point where the sketch can steer SLRU movement — admission,
  promotion entry, promotion gating, probationary eviction, protected
  demotion — has now been touched and each is marginal-or-inert. Is there any
  structural placement left untested, or is the hit-rate axis fully mapped?
- The exit lever (v002) won this cycle; would a stronger entry gate (a floor
  that actually fires, e.g. evaluated pre-increment) reserve protected slots
  in a way the current always-true entry does not?

## Continuation

The SLRU hit-rate axis is now comprehensively mapped: admission, promotion,
victim choice, and demotion have all been probed and every lever is real but
marginal or redundant. The compounding hypothesis is refuted — sampling and
promotion-gating overlap, do not stack. Across cycles 0003–0006 the evidence
that this workload's hit-rate sits at its ceiling is now very strong. Next
cycle should stop chasing hit-rate through structural refinement and pivot
decisively to efficiency/cost dimensions; if any hit-rate probe is attempted,
make it a corrected entry-floor (one that can actually fire), not another
victim/demotion sampler.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

This cycle finally fired the two probes the prior digests had reserved: a
*corrected* pre-increment entry floor (v001), a direct-to-protected admission
placement (v002), and the long-requested efficiency pivot — a 4x-narrower
sketch leaning on conservative update (v003). v001 took the Winner slot but
only by holding exactly at parent; v002 regressed a hair and v003 regressed
measurably. No lever produced lift.

## Patterns

- The corrected entry floor now engages mechanically, settling cycle 0006's
  inert-gate ambiguity — but it gates nothing on this trace: the probationary
  keys that earn a warm-up hit have already been seen twice, so the floor
  admits exactly the keys the unconditional parent admitted. A live lever with
  no work to do is the cleanest confirmation yet that the hit-rate ceiling is
  structural, not a missed gate.
- Steering placement from the *admission* side (v002) is net negative: a single
  sketch estimate is a weaker signal than an observed in-cache hit, so bypassing
  probation admits occasional false-hot keys. The probationary "earn a hit"
  filter is doing real work and removing it costs more than it saves.
- The efficiency pivot has a hard floor: conservative update does *not* hold
  accuracy at 1/4 width. The extra sketch width was not waste — this workload
  genuinely needs more cells than capacity×1 to separate its distinct keys, so
  the cheapest-cache-for-free route via narrowing is closed.

## Open questions

- With admission, promotion entry/gating, eviction, demotion, *and* a working
  entry floor all now probed and inert, is there any hit-rate lever left
  untouched, or is the structural axis fully and finally mapped?
- v003 refutes 4x narrowing but not the existence of *some* width below the
  current setting that still holds accuracy — is there a shallower memory
  reduction (e.g. 2x) that trades cells for cost without losing hits?

## Continuation

The hit-rate axis is now exhaustively mapped and confirmed at its ceiling: even
a correctly-firing entry floor finds nothing to gate, and admission-side
placement actively hurts. Stop proposing structural hit-rate refinements — every
SLRU steering point is spent. The efficiency pivot is now live but bounded: 4x
narrowing is refuted, so do not retry aggressive width cuts; if cost is pursued,
probe a *shallow* reduction (2x) to find where accuracy actually breaks, or turn
to other cost/efficiency dimensions entirely rather than memory width.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
