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
