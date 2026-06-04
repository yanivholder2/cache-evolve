# Cross-cycle insights

_Only one cycle has completed so far; treat these as provisional signals
to confirm or refute as more cycles land._

## Recurring wins

- Strict move-to-back-on-hit (the ClassicLRU baseline) has not yet been
  beaten — it remains on the Pareto front while every tested in-bucket
  tweak regressed.

## Recurring failures

- Mutations that only reorder the existing bucket without adding any
  new per-item state have so far failed to clear the baseline.
- Reusing the lifetime `lfu_counter` as a warmth signal overstated
  current relevance and weakened the cold end of the bucket.
- Slow promotion schemes (e.g. one-slot bubble-back) failed to
  concentrate hot items at the back fast enough to match strict
  move-to-back.
- Hypotheses that overload the cohort budget without producing a
  measurement (e.g. the segmented protected/probationary split) leave
  their direction untested and waste a slot.

## Open questions

- Does a segmented protected/probationary bucket actually help, once
  it is implemented within budget? It has been proposed but never
  measured.
- Can a decayed or windowed frequency signal — rather than a raw
  lifetime counter — make a cold-front frequency scan competitive on
  recency-dominated traces?
- Is there a faster-than-one-slot promotion (jump-halfway, or
  bubble-back combined with a front-end tiebreaker) that preserves
  eviction quality without paying an O(n) shift?
- The current fitness vector appears dominated by hit-rate; plausible
  per-op-time improvements went unrewarded. Are speed/quality trades
  scoreable at all under the present axes?

## Per-policy notes

- **ClassicLRU**: Move-to-back at position 0 is the bar; all
  in-bucket variants tried so far regressed hit-rate on
  `lirs:multi1`. Promising next directions are co-evolving `Item`
  metadata or bucket geometry rather than pure reorder tweaks.
