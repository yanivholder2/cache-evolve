## Recurring wins

- Eviction-side discrimination beats access-path changes. The durable
  champion (cycle 0001 v003) and the only refinement that ever beat it
  (cycle 0005 v001) both keep LRU ordering intact and only bias victim
  selection.
- Treating `lfu_counter` as a binary "ever touched" predicate is the
  one reading of the signal that has consistently paid off; richer
  readings (rank, decay-by-hit, regime counts) keep losing.
- Metadata-flat per-bucket state is a proven vehicle. Cycle 0004's
  cursor and cycle 0005's bitmask both demonstrated that per-bucket
  payloads can be added without growing `Item`, even when the payload
  itself didn't win.
- Aging works when it is event-conditional on the parent filter having
  demonstrably failed, rank-preserving, and manufactures exactly one
  fresh zero-counter victim — not when it runs on hit, on cadence, or
  by mass-clear.

## Recurring failures

- Withholding or shortening promotion inside a ~16-slot bucket: there
  is no headroom for a hot newcomer to survive its first-hit window
  (cycle 0001 v001, cycle 0002 v003).
- Replacing exact LRU ordering with coarser recency (CLOCK-style
  rotation, cursor-based fairness): strips information the existing
  index ordering was already carrying (cycle 0001 v002, cycle 0004
  v002).
- Letting derived counter signals override either `lfu_counter == 0`
  or LRU position — counter-rank, min-counter, decay-by-hit, front-half
  gating, count-based regime detection — every variant on this theme
  has lost (cycles 0002, 0003).
- Assuming axes are orthogonal when they actually couple through
  move-to-back. Insertion placement, scan direction, and fairness
  rotation all turned out to share hidden coupling with the parent's
  filter (cycle 0004).
- Unconditional or cadence-driven aging (mass-clear, K-th-eviction
  newest-kill): destroys protection for hot slots without supplying a
  replacement signal (cycle 0005 v002, v003).

## Open questions

- Does iterating halving on successive failed scans within one
  eviction call extend cycle 0005 v001's win, or is one step the right
  dose per regime entry?
- Is rank-preserving halving load-bearing on the specific shift
  operator, or would subtract-min / saturating-decrement work
  equivalently?
- Are there non-aging payloads (per-bucket insertion epoch, fresh-slot
  tag) that can ride the proven metadata-flat per-bucket vehicle and
  compose with halving rather than fight the parent filter?
- Has the existing `Bucket` / `EvictionPolicy` trait surface been
  effectively saturated? Multiple cycles have flagged widening the
  trait (insertion tick, bucket-side reference bit independent of slot
  position) as the next axis if policy-level refinement stalls again.
- Is hit-rate near a local optimum on this trace such that future
  attention should shift to the metadata-size axis?

## Per-policy notes

- ClassicLRU: move-to-back is the silent backbone of every winning
  signal. The bucket's slot ordering itself encodes age and freshness,
  which is why "orthogonal" axes (insertion placement, scan direction,
  cursor fairness) keep coupling back into the parent's filter and why
  leftmost-first scan over untouched slots is a free age signal. Any
  policy that disturbs move-to-back dynamics loses the signals other
  components silently depend on.
- CLOCK / reference-bit variants: every attempt to substitute a
  reference bit (or rotation) for exact LRU ordering on this trace has
  regressed hit-rate. The two-level recency is too coarse for a
  ~16-slot bucket; reference-bit ideas have only worked when layered
  on top of LRU as an eviction-side filter, never as a replacement for
  ordering.
- LFU-flavored signals: raw `lfu_counter` is only trustworthy as
  zero/non-zero. Rank-based and frequency-decay readings have all
  regressed; the only successful counter-mutation across five cycles
  is regime-boundary halving (cycle 0005 v001), which preserves rank
  and fires only when the binary predicate has run out of zeros.
