# Cycle 0004 Insight

## Cohort outcome

All three variants attacked cycle/0001/v003 along axes the prior digest
flagged as still-unexercised — insertion placement (v001), cursor-style
fairness among the filter's candidates (v002), and reversed scan
direction (v003) — and every one regressed hit-rate against the
parent. v002 reached the Pareto front only by spending the regression
on per-bucket cursor state that kept `Item` metadata flat.

## Patterns

- Each variant independently revealed that v003's apparent defaults
  were doing load-bearing work. Push-to-back insertion was the grace
  period that let a new key survive long enough to bump its counter.
  Leftmost-first scan was a free age signal under move-to-back
  dynamics, with the oldest never-hit slot landing at index 0. Both
  the placement axis and the scan-direction axis turn out to be
  *coupled* to the `lfu_counter == 0` filter through ClassicLRU's
  move-to-back, not independent of it.
- The cycle 0003 v002 autopsy claim that back-of-bucket untouched
  slots are predominantly one-shot churn was falsified twice in one
  cohort — once by v003's direct test (reverse scan) and once by
  v001's indirect test (gap-fill landing fresh items mid-bucket).
  On this trace those slots are mostly fresh arrivals waiting for
  their first hit, not stale one-shots.
- Spreading eviction pressure for fairness (v002) destroyed information
  rather than refining the parent: when the existing index ordering
  already correlates with a useful signal, CLOCK-style rotation strips
  that correlation without supplying a replacement.

## Open questions

- Are there *any* axes the existing trait surface still exposes that
  aren't coupled through ClassicLRU's move-to-back dynamics back into
  v003's filter? Every "orthogonal" axis the prior digests named has
  now turned out to share the same hidden coupling.
- Does the cohort's failure to find headroom anywhere on the current
  trait surface make it time to widen the `Bucket`/`EvictionPolicy`
  trait so genuinely new state (insertion tick, bucket-side reference
  bit) becomes expressible — the move cycle 0003 already flagged?
- Is per-bucket state that doesn't add per-item bytes (v002's cursor
  pattern) a vehicle worth keeping for *other* signals, even though
  the rotation use of it lost?

## Continuation

Cycle/0001/v003 has now survived three cycles of refinement. Every
axis previously flagged as orthogonal — counter aging, positional
gating, regime detection, insertion placement, scan direction,
cursor-based fairness — has been tried and lost, with the failures
all explained by the same root cause: ClassicLRU's move-to-back
dynamics make the bucket's slot ordering itself a signal that v003
already consumes. Drop further attempts to refine the policy within
the current trait surface. Next cycle should either (a) widen the
`Bucket` or `EvictionPolicy` trait so new per-bucket state (insertion
tick, reference bit) becomes expressible without piggybacking on
slot position, or (b) explicitly accept v003 as a local optimum on
this trait surface and shift attention to the metadata-size axis,
since hit-rate appears saturated for the signals currently available.
