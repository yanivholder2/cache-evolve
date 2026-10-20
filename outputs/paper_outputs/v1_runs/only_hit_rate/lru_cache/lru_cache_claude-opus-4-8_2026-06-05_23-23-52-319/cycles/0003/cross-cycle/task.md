# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants attacked the same weakness of pure LRU — it evicts a
reused key as readily as a one-hit-wonder — each via a distinct mechanism
(two-segment SLRU, midpoint insertion, CLOCK reference bit). Every variant
lifted hit-rate on this reuse-heavy LIRS trace; the segmented SLRU design
(v001) landed in the Winners.

## Patterns

- The winning lever was eviction *ordering*, not extra residency: every
  variant kept the capacity bound fixed and gained purely by deciding
  which resident key dies first.
- A "second access earns protection" signal — whether encoded as a
  protected segment, a position above the insertion midpoint, or a
  reference bit — is the common thread, suggesting the trace rewards
  distinguishing one-hit-wonders from genuine reuse.
- Cheaper, single-list approximations (midpoint insertion, CLOCK) also
  helped, so the gain does not strictly require two-segment bookkeeping —
  but the explicit segmentation won this cohort.

## Open questions

- Did SLRU win on margin, robustness, or both versus the lighter-weight
  single-list variants? The mechanism that scored highest is not
  obviously the one with the best cost/benefit.
- The 80/20 protected split in v001 was assumed, not tuned; the
  sensitivity of hit-rate to that ratio is unexplored.
- Whether the three protection signals are additive (e.g. segments +
  reference bit) or redundant is untested.

## Continuation

Pursue the "protect on second access" family — it is clearly the right
axis for this workload. Next cycle should tune within the winner (probe
the protected/probationary ratio and the promotion threshold) rather than
re-deriving that segmentation beats LRU. Consider one variant that
combines a cheap signal (reference bit or midpoint) with segmentation to
test additivity, and drop any further pure-recency baselines.


### Cycle 0002

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


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

This cycle pursued the three orthogonal levers cycle 0002 left open:
push the protected ratio further (H01, 90/10), frequency-windowed
eviction-victim selection (H02), and ghost re-admission with *weak*
(protected-LRU) protection (H03). Only the ratio push landed a clear
gain — v001 (90/10) won. The two scan-resistance / reuse-distance levers
worked directionally but barely moved hit-rate above noise.

## Patterns

- Protected residency remains the dominant lever. Confirming cycle 0002's
  predicted direction, enlarging the protected tier to 90/10 still beats
  the parent — more room for twice-touched hot keys converts directly to
  hits, and a 205-slot probationary segment still drains one-hit-wonders.
- The cleverer eviction-order machinery (frequency window, ghost memory)
  is nearly inert on this trace because second-access promotion already
  captures almost all the reuse signal *before* eviction order matters.
  Both H02 and H03 only rescue a thin band of warm-but-unpromoted keys.
- Cycle 0002's ghost placement diagnosis was correct: routing ghost hits
  to the protected *LRU* end (v003) recovers signal without crowding the
  hot set, where the prior protected-MRU placement regressed — but the
  recoverable headroom here is genuinely small.

## Open questions

- Is 90/10 the peak, or does the protected tier keep paying past it
  (95/5)? The probationary segment must stay non-empty for the eviction
  path, so there is a hard ceiling somewhere.
- The two near-flat levers (frequency window, weak-protection ghost) were
  tested in isolation; would either contribute once the ratio is at its
  optimum, or are they redundant with a maximally-protected tier?
- With promotion and ratio both near-tuned, has this trace's exploitable
  structure been largely consumed?

## Continuation

Keep pushing protected residency — probe past 90/10 (e.g. 95/5) to find
where the probationary floor breaks the eviction path; that is the live
lever. Treat frequency-windowed eviction and ghost re-admission as
confirmed-marginal on this trace and drop them as standalone probes. If
anything, test whether either adds value layered on the 90/10 winner
rather than on the original 80/20 parent. The cohort increasingly looks
near its ceiling on recency/promotion axes; consider whether a
fundamentally different workload signal remains untapped.


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
