# Cycle 0005 Insight

## Cohort outcome

Three variants attacked cycle/0001/v003's all-touched fallback with
three different aging/scan-resistance mechanisms — on-demand counter
halving at the regime boundary (v001), a per-bucket reference bitmask
with mass-clear aging (v002), and a fixed-cadence newest-slot sacrifice
(v003). v001 was the only one to make the Pareto front; v002 and v003
both regressed hit-rate against the parent.

## Patterns

- Aging finally paid off, but only as a *regime-boundary* operator,
  not as a periodic or amortized one. v001 fires the halving step
  exclusively at the exact moment v003's untouched-first scan fails,
  preserving the `lfu_counter` signal everywhere else; v002's mass-
  clear on the same trigger destroys the durable "ever touched" bit
  for every hot slot simultaneously, and v003's K-cadence newest-kill
  fires regardless of whether the bucket is actually in the regime it
  was designed for. The cohort sharpens the rule from cycle 0003: aging
  must be *event-conditional on the parent's filter already having
  failed*, not driven by hits, sweeps, or eviction count.
- Right-shift halving turns out to be the minimum-information aging
  operator that respects v003's invariants. It manufactures a fresh
  zero-counter victim (the previous lowest-counter slot) while keeping
  rank order between long-lived items intact — neither erasing
  protection (v002) nor overriding the filter with an unrelated signal
  (v003). This is the first time across five cycles that adding
  state-mutation on top of v003 has improved hit-rate rather than
  regressed it.
- v003's parent-overlay design surfaced again as load-bearing. v002 and
  v003 both reused the metadata-flat per-bucket vehicle that cycle
  0004's cursor variant proved out, but neither could turn that
  vehicle into a hit-rate win because the payload they ran on it
  fought v003's filter rather than extended it. The vehicle is
  proven; the signals that ride on it still have to compose with —
  not replace — `lfu_counter == 0`.

## Open questions

- Now that on-demand halving works at the all-touched boundary, does
  iterating the same halving (or a deeper shift) on *successive*
  failed scans within the same eviction call open further headroom, or
  is one halving step the right amount of aging per regime entry?
- Is the win specific to halving, or would any rank-preserving
  monotone-decreasing transform that produces a zero (subtract-min,
  saturating decrement) work equally well? The cohort exercised only
  one operator on this axis.
- v002 confirmed the per-bucket vehicle stays metadata-flat under a new
  payload. Are there *non-aging* signals (per-bucket insertion epoch,
  fresh-slot tag) that could ride the same vehicle and compose with
  v001's halving, now that the regime-boundary aging path is no longer
  the bottleneck?

## Continuation

Cycle/0005/v001 is the new champion: it is the first variant in five
cycles to exceed cycle/0001/v003 on hit-rate while staying metadata-
flat, and it does so by adding aging in the one form the prior digests
kept circling — event-conditional, rank-preserving, fired only when
v003's filter has demonstrably failed. Next cycle should treat this as
the parent. Drop further attempts at hit-frequency decay, mass-clear
aging, and unconditional cadence overrides — this cohort independently
falsified each variant. Promising directions: (a) explore whether a
second or deeper halving on repeated failed scans within one eviction
adds headroom, (b) test alternative rank-preserving aging operators
against halving to confirm it is not load-bearing on the specific
shift, (c) revisit the metadata-flat per-bucket vehicle for non-aging
signals that could compose with v001's halving rather than replace
v003's filter.
