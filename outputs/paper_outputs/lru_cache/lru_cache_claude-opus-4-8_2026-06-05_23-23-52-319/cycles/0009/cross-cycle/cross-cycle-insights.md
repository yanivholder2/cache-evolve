## Recurring wins

- Eviction *ordering*, not extra residency, is the core lever: gains came
  from deciding which resident key dies first under a fixed capacity
  bound. "Protect on second access" (SLRU/midpoint/CLOCK) beat plain LRU
  decisively and remains the productive axis.
- Enlarging the protected tier kept paying across four cycles
  (80/20 → 90/10 → 95/5), each step a real but shrinking win — more room
  for twice-touched hot keys converts directly to hits.
- First-hit promotion plus a generous protected tier is confirmed
  correct: reuse on this trace arrives fast, within the narrow
  probationary window.

## Recurring failures

- Loosening the promotion signal always regresses: two-hit gates, wider
  probationary dwell, and post-eviction straight-to-protected ghosts all
  lost keys that churn out before a delayed second touch.
- Non-recency signals lose inside the SLRU frame every time: LFU victim
  selection, frequency at the demotion point, and front admission windows
  (W-TinyLFU) are inert or harmful — one-hit-wonders do not dominate this
  reuse-heavy trace.
- Ghost / reuse-distance re-admission is structurally redundant against a
  maximally-protected tier: the warm-but-evicted band left to rescue is
  vanishingly thin, and it costs structure plus per-insert scans.
- Drain-floor protection is exhausted in both skeletons: static window,
  reprieve laps, adaptive boundary resizing, and reactive minimums all
  fail — the binding constraint is the probationary floor's *size*, which
  these erode.
- SLRU tuning lessons do not transplant onto LIRS as parameter edits;
  cross-policy transfer is disconfirmed.

## Open questions

- The persistent blind spot: every new skeleton (LIRS, ARC, 2Q) is
  parented on baseline and reports a baseline-relative lift, but no clean
  head-to-head against the 95/5 SLRU incumbent on its home trace has ever
  been run. Does any of them actually clear the incumbent?
- Is ARC's residual gap tunable via T2 internal ordering/admission (its
  size lever is a confirmed no-op), or is the skeleton simply below the
  incumbent?
- Has this trace's recency/promotion structure been fully consumed, or
  does a genuinely different workload signal remain in an alternative
  skeleton?

## Per-policy notes

- **SLRU**: converged at 95/5. Geometry is closed — static tuning,
  adaptive resizing, depth-stratified protected tiers, and every
  ghost-placement variant have been tested and exhausted.
- **LIRS**: reported a real lift on its home turf but never cleanly
  verified against the incumbent; all tuning attempts (HIR window, ghost
  diversion, widened horizon, reactive floor) regressed.
- **ARC**: first skeleton to beat baseline structurally via
  recency/frequency separation; `p`-clamp tuning is a no-op because `p`
  never climbs on this trace. Untested directly against incumbent.
- **2Q**: FIFO admission filters one-hit-wonders and beats baseline; like
  LIRS and ARC, never raced against the 95/5 incumbent.
