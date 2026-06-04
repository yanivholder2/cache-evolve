# Cycle 0003 Insight

## Cohort outcome

All three variants forked cycle/0001/v003 and attacked exactly the three
axes the prior cycle's open questions named — on-hit decay (v001),
positional gating of the untouched-first scan (v002), and a cardinality
proxy for the all-touched regime (v003) — and every one of them
regressed hit-rate against the parent. Only v003 reached the Pareto
front, and only by being the least costly regression while staying flat
elsewhere.

## Patterns

- The intuitive readings of each signal all inverted on this trace.
  Decay was supposed to convert lifetime hits into recent-relative
  protection but instead drained genuine re-referenced items back to 0.
  Back-half position was supposed to mark "hot newcomer" but in
  practice marked "freshly-inserted one-shot." A lone untouched slot
  was supposed to be the freshly-inserted keeper but in churn-heavy
  buckets is just as often a stale one-shot the parent's scan was
  correctly killing.
- v003's untouched-first scan is robust because it commits to a binary
  signal and degrades gracefully when no untouched slot exists. Every
  refinement this cycle tried — temporal (decay), spatial (front-half),
  or population-based (count >= 2) — replaced correct evictions in
  some sub-regime with incorrect ones, with no offsetting gain in the
  regimes those refinements were aimed at.
- The cohort confirms cycle 0002's diagnosis from a new angle: any
  policy that lets a derived signal override either the raw
  `lfu_counter == 0` predicate or LRU position loses on this workload.
  The headroom inside a ~16-slot bucket is too narrow for the parent's
  zero-information fallback to be the wrong default.

## Open questions

- Are there signals outside `lfu_counter` + bucket position — e.g., a
  per-slot insertion tick, or bucket-level admission state — that could
  distinguish "fresh insert" from "stale one-shot" without the failure
  modes this cycle hit? The cohort never reached this axis because the
  trait's `&[Item]` eviction signature still constrains side-channel
  state on the bucket.
- Have we exhausted the eviction-side axis entirely on this trace, or
  does composing v003's filter with an orthogonal *insertion-placement*
  axis (where new items land in the bucket, not which slot is evicted)
  still have headroom?
- Is cycle/0001/v003 close to a local optimum within the current trait
  surface, such that further wins require widening the trait rather
  than refining the policy?

## Continuation

Cycle/0001/v003 has now survived two cycles of attempted refinement
along every axis the prior digests flagged as promising. Drop further
variations on on-hit decay, positional gating of the eviction scan, and
cardinality-based regime detection — this cohort independently
falsified each of the three open questions cycle 0002 left open. Next
cycle should either (a) move off the eviction-side axis entirely and
explore insertion placement or per-slot age signals that the existing
trait can carry, or (b) explicitly widen the bucket trait so genuinely
new state (insertion tick, reference bit on the bucket itself) becomes
expressible — every refinement of the existing signal space has now
underperformed the parent.
