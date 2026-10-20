# Cycle 0002 Insight

## Cohort outcome

This cycle probed three independent tweaks to last cycle's SLRU winner —
widen the probationary segment (H01), add a ghost list so post-eviction
reuses skip probation (H02), and require two hits before promotion (H03).
All three hurt hit-rate; none beat the parent. v003 (two-hit promotion)
landed in the Winners as the least-regressive, but every probe refuted its
own hypothesis.

## Patterns

- The parent's tuning was already near-optimal for this trace, and it
  errs in one consistent direction: **promote early, protect generously.**
  Loosening either lever lost hits — shrinking protected residency (H01),
  delaying promotion to a second hit (H03), and over-trusting a single
  post-eviction reuse (H02) all backfired.
- Reuse on this trace arrives *fast*: the second touch lands within the
  narrow 20% probationary window, so extra probationary dwell time (H01)
  is wasted and a stricter two-hit gate (H03) just loses keys that churn
  out before their second access.
- The binding constraint is protected-tier residency, not the strength or
  timing of the promotion signal. Anything that steals protected capacity
  or admits marginal keys into protected (H02's straight-to-protected
  ghost promotion) demotes genuinely hot residents and costs hits.

## Open questions

- The ratio should move *opposite* to H01 — does a larger protected tier
  (e.g. 90/10) keep winning, or is 80/20 already at the peak?
- Ghost/reuse-distance memory was directionally right but mis-placed:
  would routing ghost hits back to *probationary* (not protected) recover
  the signal without crowding the hot set?
- Every lever tested so far is monotone-worse than the parent; is v001's
  configuration a genuine local optimum, or are these three axes simply
  the wrong ones?

## Continuation

Stop loosening the parent — first-hit promotion and an 80/20-or-larger
protected tier are confirmed correct; drop further two-hit-gate and
wider-probationary probes. Next cycle should push protected residency the
*other* way (probe 85/15, 90/10) and, if revisiting ghosts, demote the
promotion target to probationary rather than protected. Treat v001 as a
strong local optimum and look for orthogonal levers (eviction-victim
selection, scan resistance) rather than re-tuning the promotion signal.
