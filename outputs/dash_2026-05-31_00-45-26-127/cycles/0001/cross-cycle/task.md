# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

This cohort probed three orthogonal in-bucket tweaks to ClassicLRU — a
segmented protected/probationary split (H01, dropped to an overload),
a one-slot bubble-back swap (H02), and a frequency-weighted victim
scan over the cold quartile (H03). Only the baseline survived on the
Pareto front; both completed mutations regressed hit-rate on
`lirs:multi1` and neither bought back a measurable win on any other
fitness axis.

## Patterns

- Both completed variants weakened the cold end of the bucket: H02 by
  promoting too slowly to concentrate hot items at the back, H03 by
  protecting items whose lifetime counter overstated their current
  warmth. Strict move-to-back at position 0 is harder to beat than it
  looks on this trace.
- The only fitness signal that moved was hit-rate. H02's plausible
  per-op-time improvement was invisible to the fitness vector, so
  speed/quality trades can't be scored this cycle — every change
  collapses to "does it raise hit-rate."
- Both mutations stayed inside `eviction_policy.rs` and reused
  existing item metadata. No variant explored adding state to `Item`
  or co-evolving the bucket layout, even though H01 (which would
  have) never produced a measurement.

## Open questions

- Would a segmented-bucket scheme (H01's untried direction) survive
  where bubble-back and stale-counter LFU did not, by protecting
  re-referenced items without relying on a lifetime counter?
- Can a recency-bounded or decayed frequency signal — rather than
  raw `lfu_counter` — make a cold-front frequency scan competitive
  with blind front-pop on a recency-dominated trace?
- Is there a faster-than-one-slot promotion (e.g. jump halfway to the
  back, or bubble-back combined with a front tiebreaker) that
  preserves eviction quality while avoiding the O(n) shift?

## Continuation

Next cycle should retry the segmented-bucket idea (H01) since this
cohort never measured it, and pair it with frequency-signal designs
that decay or window the hit count rather than reusing the lifetime
`lfu_counter` as-is. Drop pure one-step bubble-back: a cheaper
promotion needs to move warm items faster than a single slot per hit,
or combine with a front-end tiebreaker, before it can match strict
move-to-back on `lirs:multi1`. Hypotheses that only touch eviction
order without adding any new per-item state are unlikely to clear the
baseline — co-evolving `Item` or bucket geometry is the more
promising direction.


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
