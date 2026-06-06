## Recurring wins

- Eviction *ordering*, not extra residency, is the lever that pays: every
  gain has come from deciding which resident key dies first under a fixed
  capacity bound, never from relaxing the bound.
- "Protect on second access" is the durable winning signal — distinguishing
  one-hit-wonders from genuine reuse (via segments, midpoint, or reference
  bit) beat pure LRU in the first cohort and anchored every cycle since.
- Enlarging the protected tier has paid repeatedly (80/20 → 90/10 → 95/5),
  the most reliable monotone lever across cycles — though each step's return
  shrank and the latest win was the thinnest yet.
- First-hit (early, generous) promotion is confirmed correct: reuse on this
  trace arrives fast, landing within a narrow probationary window.

## Recurring failures

- Loosening the parent's promotion discipline always regressed: two-hit
  promotion gates and wider probationary segments both lose keys that churn
  out before a delayed second touch.
- Ghost / reuse-distance re-admission is persistently marginal-to-dead:
  directionally right but structurally redundant against a heavily protected
  tier, where the warm-but-evicted band left to rescue is vanishingly thin —
  tested in isolation and layered on the incumbent, it never moved the needle.
- Non-recency admission signals (front admission window, W-TinyLFU) are
  actively harmful: their windows are far smaller than this trace's reuse
  distances, evicting genuinely reused keys before their second touch.

## Open questions

- Is the protected ratio at its ceiling? Each push still pays a little, but
  the probationary floor needed to drain one-hit-wonders sets a hard limit
  somewhere near the current point.
- Has the recency/promotion axis been fully consumed, leaving the incumbent
  a converged optimum — or is there a genuinely different, untapped workload
  signal that off-recency levers keep failing to capture?
- Are the cheap protection signals (reference bit, midpoint) additive with
  segmentation, or redundant? Never tested in combination.

## Per-policy notes

- Pure LRU: baseline only; evicts reused keys as readily as one-hit-wonders
  and has been beaten in every cohort. Dropped as a probe.
- SLRU (segmented): the consistent winner and incumbent; gains track
  protected-tier size, with promotion timing already near-optimal.
- Midpoint insertion / CLOCK: cheap single-list approximations that also beat
  LRU in cohort 1, showing the gain doesn't strictly require two-segment
  bookkeeping — but explicit segmentation has won every head-to-head.
- Ghost-list and admission-window variants: confirmed marginal or harmful on
  this reuse-heavy LIRS trace; retired as standalone and layered probes.
