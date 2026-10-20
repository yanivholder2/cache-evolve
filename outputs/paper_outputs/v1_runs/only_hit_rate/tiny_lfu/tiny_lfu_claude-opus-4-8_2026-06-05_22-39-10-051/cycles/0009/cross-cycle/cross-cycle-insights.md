# Cross-Cycle Insights

## Recurring wins

- Sharpening the *estimator* paid off cleanly once: widening the count-min
  sketch (cycle 0002) was the single largest hit-rate lever found. It then
  plateaued and never reversed.
- Spending the existing frequency signal carefully on *victim choice* won the
  first cycle (0001) and remained directionally positive wherever the sketch
  steers SLRU movement (eviction, demotion) — but only ever at the noise floor.
- *Tightening* admission/promotion gates is the one direction still nudging
  upward late in the search (cycles 0007–0009): self-consistent (pre-increment)
  scoring and a hysteresis margin both edged past parent by sparing warm
  residents from marginal, non-reused arrivals.
- An SLRU split with a *large* protected tier is the best structural substrate;
  larger protection beats smaller, and the favored direction has held since
  cycle 0005.

## Recurring failures

- Any mechanism that discards, resets, or withholds long-term frequency signal
  loses: sketch aging, an unconditional admission window (0001), and the
  doorkeeper across multiple cycles (0002–0003).
- *Loosening* gates or steering from the admission side consistently costs
  hits (0007–0009): a single sketch estimate is a weaker signal than an
  observed in-cache hit, so bypassing the "earn a probationary hit" filter
  admits false-hot keys. Strict guards keep winning.
- Overlapping structural mechanisms do not stack: victim sampling and
  frequency-gated promotion fire on the same rare "head hides a hotter key"
  events and net to nothing when combined (cycle 0006).
- Aggressive estimator narrowing is refuted: 4x-narrower sketch + conservative
  update cannot hold accuracy — this trace genuinely needs more than capacity×1
  cells (cycle 0007).
- Dynamic/adaptive segment sizing does no better than static sweeps and
  slightly worse — there are no phases worth tracking (cycle 0009).

## Open questions

- The efficiency/cost pivot has been reserved for many cycles but repeatedly
  skipped: depth was proven redundant (0008) and protected sizing flat above
  the favored point — can depth or protected slack be *trimmed for free*
  without losing the ceiling hit-rate? A shallow (2x) width cut is also
  untested between current width and the refuted 4x.
- Do the two late gate tightenings (self-consistent scoring + hysteresis
  margin) stack, or do they consume the same handful of tie contests the way
  sampling and promotion-gating overlapped?
- Is hit-rate at this capacity conclusively capped? Every estimator and SLRU
  steering point has been probed and found marginal-or-inert.

## Per-policy notes

- The search has converged on one family: W-TinyLFU / count-min sketch over an
  SLRU (protected + probationary) main space, on a single popularity-stable,
  low-churn workload.
- Estimator axis is saturated on *both* dimensions — width (0002–0007) and
  depth (0008) — collisions are already removed; further fidelity buys nothing.
- SLRU steering is comprehensively mapped — admission, promotion entry,
  promotion gating, probationary eviction, protected demotion, and protected
  sizing — every point is real but marginal or redundant, because recency order
  within both segments already tracks sketch frequency closely.
- Workload signature: a lever only matters when the LRU head hides a key much
  colder or hotter than the gate already detects; that case is rare here, which
  is why every structural win hovers at the noise floor.
