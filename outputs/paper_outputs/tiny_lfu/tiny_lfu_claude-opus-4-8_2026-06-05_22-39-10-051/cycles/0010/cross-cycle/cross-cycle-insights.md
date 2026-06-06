# Cross-Cycle Insights

## Recurring wins

- Spending more on **estimator fidelity** was the one early lever that lifted
  hit-rate: widening the count-min sketch separated the trace's distinct keys
  and won cleanly (cycle 0002). Every later hit-rate gain has been at the noise
  floor by comparison.
- **Strict guards win, loose guards lose.** Refinements that *tighten* who may
  evict or be promoted — frequency-ranked victim choice, promotion gating,
  protected-tier demotion choice, self-consistent pre-increment scoring,
  hysteresis margin — are consistently directionally positive (cycles 0001,
  0005, 0006, 0009), even when the margin is sub-noise.
- The SLRU **protected tier wants to be large** (≥0.8); enlarging probation
  refuted (cycle 0005), and sizing is a flat plateau at the top (0.8–0.9 tie,
  cycle 0008).
- The **efficiency pivot pays for free**: the sketch was over-provisioned on
  width and can be halved (64x→32x) holding hit-rate flat while reclaiming
  memory (cycle 0010).

## Recurring failures

- **Discarding or withholding long-term frequency signal** never pays:
  aggressive sketch aging, unconditional admission window, and the admission
  doorkeeper all lost or regressed (cycles 0001–0003). This is the most durable
  negative across the whole run.
- **Loosening admission / bypassing probation** loses: admission-side direct-
  to-protected placement and gate-loosening both regress because a single
  sketch estimate is weaker than an observed in-cache hit (cycles 0007–0008).
- **Over-spending the signal** plateaus then reverses: deeper victim sampling
  and the doorkeeper trample recency-warm or warming keys before counts
  accumulate (cycle 0002).
- **Overlapping mechanisms do not stack**: victim sampling + promotion gating
  (cycle 0006) and the two gate tightenings (cycles 0009–0010) fire on the same
  scarce contests, netting to nothing or over-correcting into loss.
- **Adaptive segment sizing** is no better than static and slightly worse —
  there are no phases worth tracking on this trace (cycle 0009); the smaller-
  protected-fraction direction is refuted (cycle 0005).
- **Aggressive memory cuts break accuracy**: 4x sketch narrowing and depth cut
  below 4 both regress (cycles 0007, 0010).

## Open questions

- Width holds flat at 32x; is there a further free cut (16x, 8x) before
  accuracy actually breaks, or is 32x near the floor?
- The fitness vector is hit-rate-only and **cannot score memory/cost savings** —
  every efficiency trim registers as a tie at best. Is there a path to credit
  cost directly, or will future efficiency work keep being judged on flatness?
- With estimator (width+depth), every SLRU steering point, segment sizing, and
  gate tightening all probed and inert, is *any* hit-rate lever left, or is the
  trace conclusively at its ceiling?

## Per-policy notes

- **Count-min sketch (estimator):** the single source of real hit-rate lift
  (width, cycle 0002), now saturated on both width and depth (cycles 0003–0008).
  Over-provisioned on width — trimmable to 32x for free — but depth-4 is the
  redundancy floor; conservative update is redundant with width and cannot hold
  accuracy at reduced width.
- **W-TinyLFU window / doorkeeper / sketch aging:** all refuted early —
  mechanisms that assume shifting popularity or burst protection find no problem
  to solve on this popularity-stable, low-churn trace.
- **SLRU (protected/probationary):** the live structural substrate. Every
  steering point (admission, promotion entry/gating, probationary eviction,
  protected demotion) probed; each real but marginal. Favors a large protected
  tier; dynamic sizing inert. Strict gates are the only positive direction,
  always sub-noise.
- **Overall workload signature:** popularity-stable, low-churn. Recency order
  already tracks sketch frequency closely, so frequency contests rarely reverse
  a decision — hit-rate is at its ceiling and the search has pivoted to
  free-trim efficiency.
