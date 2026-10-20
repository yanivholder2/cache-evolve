# Cross-Cycle Insights

## Recurring wins

- Eviction *ordering*, not extra residency, is where gains come from: every
  win kept the capacity bound fixed and changed only which resident dies first.
- "Protect on second access" / first-hit promotion is the load-bearing signal
  on this trace — reuse arrives fast, so a single early touch is the right
  promotion trigger.
- Protected-tier *size* is the dominant lever: successive enlargements of the
  protected segment (80/20 → 90/10 → 95/5) each won, with steadily shrinking
  margins as the probationary drain floor became the binding constraint.
- When a probe must regress, the variant anchored to the incumbent lineage
  tends to take the Winners slot by regressing least — many recent "wins" are
  confirmations of a negative.

## Recurring failures

- Loosening the promotion signal (two-hit gate, wider probationary dwell) keeps
  losing keys that churn out before a second touch.
- Non-recency signals inside the SLRU frame — LFU victim selection, frequency
  at the demotion point, front admission windows (W-TinyLFU) — regress or go
  inert; one-hit-wonders do not dominate this LIRS-derived trace.
- Reuse-distance memory (ghost lists, re-admission, reactive/adaptive floors)
  is directionally right but redundant against a near-fully-protected tier; the
  warm-but-evicted band is too thin to monetize.
- Adaptive geometry (floating the protected/probationary split) does worse than
  the static optimum; the split cannot be discovered from reuse feedback alone.
- Transplanting SLRU tuning lessons onto LIRS is disconfirmed — lessons inside
  one skeleton are not portable as parameter edits to a different invariant
  structure.
- Every alternative skeleton (LIRS, ARC, 2Q) beats the plain-LRU baseline
  structurally, yet none clears the tuned 95/5 SLRU incumbent when raced.

## Open questions

- Is there any genuinely untested *structural* axis left, or is the 95/5 SLRU
  optimum demonstrably unbeatable on this trace?
- ARC's residual gap was never decomposed to a single mechanism the way 2Q's
  was — its T2 internal ordering/admission is the one unfinished thread.
- A persistent process blind spot (now mostly closed): new skeletons were
  parented on baseline and reported baseline-relative lifts; direct
  head-to-heads against the incumbent were deferred for many cycles.

## Per-policy notes

- **SLRU**: The converged winner at 95/5. Geometry is exhausted both statically
  (97/3 regresses below the drain floor) and adaptively; every drain-floor,
  reprieve, ghost-placement, and depth-stratification tweak has regressed.
- **LIRS**: Home-turf skeleton; reported a real lift early but a clean untuned
  LIRS-vs-incumbent verification was repeatedly called for and never executed.
  Its S-membership/stack pruning already does the filtering SLRU bolts on, so
  SLRU-style tunings (wider HIR window, weak-tier ghost diversion, widened
  non-resident horizon, reactive floor) all backfire.
- **ARC**: First skeleton to beat baseline by separating recency (T1) from
  frequency (T2). Gap to incumbent is *not* recency-tier size — clamping `p`
  was a no-op since `p` never climbs on this reuse-heavy trace; the gap lives
  in T2 ordering/admission, still undecomposed.
- **2Q**: Beats baseline via FIFO admission filtering. Raced directly and
  trailed; deficit pinned to neither geometry nor promotion timing — the
  delayed A1out ghost gate is load-bearing and demands cross-lifetime reuse
  proof before a main-list slot.
