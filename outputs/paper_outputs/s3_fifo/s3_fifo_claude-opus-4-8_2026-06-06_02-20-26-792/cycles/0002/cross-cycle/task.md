# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

This cycle probed three independent S3-FIFO levers against the baseline —
admission threshold (H01), small/main partition ratio (H02), and ghost-queue
history depth (H03). Only the partition-ratio change (v002) landed in the
Winners; loosening the promotion gate and deepening ghost history both hurt
hit-rate.

## Patterns

- The three "move toward the paper's defaults" hypotheses split sharply: the
  sizing default helped, while the paper's promote-on-any-reuse threshold and a
  wider history window both regressed. On this LIRS workload the baseline's
  stricter filtering is load-bearing, not an oversight.
- Anything that admits more keys into the main resident queue — looser
  promotion (v001) or long-gap direct re-admission via deeper ghost (v003) —
  displaced hotter residents and cost more than it saved. The main queue is
  contention-sensitive; protecting it beats feeding it.
- Conversely, giving the main queue *more capacity* (v002) was the one net win,
  but only modestly, suggesting the partition ratio is a real but bounded knob.

## Open questions

- How far can the small/main ratio be pushed before the shrinking admission
  filter starts leaking one-hit-wonders into ghost/main? v002's gain was small —
  is there a sharper optimum past ~10/90?
- v003 showed history depth and the direct-to-main re-admission rule are
  coupled. Would deeper ghost history help if ghost hits were gated (e.g.
  re-admitted via small queue, or only when fresh) rather than pushed straight
  to main?

## Continuation

Pursue the partition-ratio lever further — sweep small/main beyond 10/90 to
find where the main-queue gain reverses. Drop standalone threshold-loosening
(H01-style) and naive ghost-deepening (H03-style); both regressed because they
crowd the main queue. If revisiting ghost depth, pair it with a gate on the
direct-to-main re-admission path rather than widening history alone.


### Cycle 0002

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


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
