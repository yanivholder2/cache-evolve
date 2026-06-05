# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0005

## Winners

- v003

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0005/variants/v001/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0005/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0005/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0005/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0005/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

This cycle ran the three threads cycle 0003 left open against the 90/10
SLRU winner: push the protected ratio to 95/5 (H01), layer a
weak-protection ghost on the incumbent (H02), and add a W-TinyLFU
admission window for scan resistance (H03). Only the ratio push won —
v001 (95/5) landed in the Winners, but by the thinnest margin yet. The
ghost was barely perceptible and the admission window actively
regressed.

## Patterns

- The protected-residency lever still pays at 95/5, but its return is
  now nearly spent: shrinking probationary to ~100 slots produced a far
  smaller gain than each prior enlargement. The binding constraint has
  shifted from "appetite for protected tenure" to the probationary floor
  needed to keep draining one-hit-wonders.
- Reuse-distance memory (the ghost) is directionally right but
  structurally redundant against a maximally-protected tier: with 90% of
  capacity already protected, genuinely hot keys are retained anyway, so
  the warm-but-evicted band left for the ghost to rescue is vanishingly
  thin — and it costs a ghost structure plus an O(n) scan per insert.
- A non-recency admission signal is not just inert here but harmful: the
  ~32-slot window is far smaller than this trace's reuse distances, so it
  evicts genuinely reused keys before their second touch. Scan resistance
  assumes one-hit-wonders dominate; on this LIRS trace they do not.

## Open questions

- 95/5 still pays but barely; is there any room left above it before the
  probationary segment can no longer host the eviction path, or is this
  effectively the peak?
- Both off-axis levers (ghost, admission window) have now been tested in
  isolation *and* layered on the incumbent and failed both times — is
  there any orthogonal signal in this trace left to exploit, or is the
  recency/promotion axis fully consumed?

## Continuation

Treat the protected-ratio lever as essentially exhausted: 95/5 is the
last marginal win and probably at or adjacent to the ceiling — do not
spend a future cycle on 97/3-style probes expecting real gains. Ghost
re-admission and front admission windows are now confirmed dead on this
trace (redundant or harmful); drop both permanently. The cohort has
consumed nearly all recency/promotion structure; the next cycle should
either accept the incumbent as a converged optimum or look for a
genuinely different workload signal rather than re-tuning SLRU geometry.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
