# Cycle 0004 Insight

## Cohort outcome

With the partition ratio frozen at ~4/96, this cycle abandoned main-queue
sizing and probed three genuinely orthogonal mechanisms off the converged
parent: ghost *sizing* alone (H01), warm-starting re-admitted entries'
frequency (H02), and switching main from pure FIFO to move-to-front recency
(H03). Only the ordering-discipline change (v003) landed in the Winners, and
only by a sliver; both ghost-side levers regressed.

## Patterns

- The two ghost-side bets failed for the same root reason: a lone ghost hit
  is weak reuse evidence, yet the unguarded direct-to-main path treats every
  ghost hit as confirmed. Deepening the ghost (v001) manufactured false
  re-admissions from ancient keys; warm-starting frequency (v002) let
  single-hit keys outlast a real eviction pass. Both crowded hotter
  residents. The discriminating signal is accumulated `get()` frequency, not
  the ghost hit itself — so neither ghost depth nor re-admission protection
  is a live lever here.
- The first lever to pay anything since sizing converged was the *ordering
  discipline* of main — making the resident queue recency-aware instead of
  position-blind FIFO. This confirms the LIRS trace carries recency structure
  the partition knob never touched. But the gain was marginal: the existing
  `freq` second-chance counter was already a decent recency proxy, so explicit
  move-to-front recovers only a sliver.
- The cross-cycle pattern holds: every lever that adds protection or memory to
  weak-signal keys regresses, while levers that sharpen which residents the
  eviction scan reaches can pay. Reordering (what gets evicted) beats
  re-admitting or warm-starting (what gets kept).

## Open questions

- The move-to-front win is thin — is there more in the ordering axis (e.g.
  reordering on promotion, or recency-aware tail selection) or is this lever
  also near its floor?
- With ghost sizing, ghost re-admission, frequency warm-start, and now
  ordering all probed, is the remaining headroom on this workload small enough
  to declare the configuration converged?

## Continuation

Keep the partition ratio frozen and the direct-to-main re-admission path
intact. Drop both ghost-side axes for good — ghost *sizing* without a gate is
inert-to-harmful, and frequency warm-starting on re-admission regressed;
neither deserves another probe. The one open thread is main's ordering
discipline: v003's move-to-front paid, barely, so the next cycle should test
whether the recency axis has more to give (alternative reorder triggers or
recency-aware eviction selection) before concluding the configuration is
converged.
