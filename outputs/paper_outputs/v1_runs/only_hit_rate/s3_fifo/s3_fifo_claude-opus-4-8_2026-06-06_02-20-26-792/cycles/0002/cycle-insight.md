# Cycle 0002 Insight

## Cohort outcome

This cycle pushed three retention levers off cycle-0001's winning 10/90
parent: shrinking the admission queue further (H01), deepening the
frequency clamp (H02), and gating ghost re-admission through the small
queue (H03). Only the further partition shrink (v001, ~6/94) landed in the
Winners; deepening the second-chance ceiling barely moved, and throttling
ghost re-admission regressed.

## Patterns

- The partition ratio remains the dominant lever, but it is now visibly
  near its plateau: pushing from 10/90 to ~6/94 still paid, yet the gain
  shrank relative to the prior step. The optimum is close and the next
  shrink risks leaking one-hit-wonders rather than helping.
- Levers that target the *same* retention behavior as the partition ratio
  (raising the frequency ceiling) buy almost nothing once main is already
  large — hot keys clear the eviction gate well before the old cap bound
  them. Distinct-on-paper knobs collapse into the partition knob.
- Cycle 0001's "every direct-to-main path is contention-sensitive" finding
  does NOT generalize to the ghost re-admission path. A ghost hit is itself
  reuse evidence; demoting it to a small-queue re-confirmation discards
  that signal whenever the long-gap reaccess outlives small residency.
  Throttling here cost more than it saved.

## Open questions

- Where exactly does the partition ratio reverse? v001 is past the knee but
  still positive — one more probe (e.g. ~4/96) would locate the turn.
- With main near its useful maximum, are there levers *orthogonal* to main
  retention still untapped — e.g. ghost queue sizing/freshness, or the
  promotion gate threshold itself — rather than more ways to enlarge or
  protect main?

## Continuation

The partition-ratio sweep is nearly exhausted: take at most one more shrink
step to confirm the reversal, then freeze it. Drop frequency-ceiling
deepening (redundant with a large main) and ghost re-admission gating
(regressed — keep the direct-to-main re-admission path intact). Next cycle
should turn to levers orthogonal to main-queue retention, since stacking
more retention onto an already-large main is yielding diminishing returns.
