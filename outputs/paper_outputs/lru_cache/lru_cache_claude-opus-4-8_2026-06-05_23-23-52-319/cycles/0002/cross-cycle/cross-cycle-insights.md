## Recurring wins

- Eviction *ordering*, not extra residency, is the lever that pays: gains
  come from deciding which resident dies first while the capacity bound
  stays fixed.
- Distinguishing genuine reuse from one-hit-wonders is the workload's
  central reward — a "second access earns protection" signal helps however
  it is encoded (segment, midpoint, reference bit).
- Promote early and protect generously: first-hit promotion plus a large
  protected tier (80/20 or more) is the configuration that keeps winning.

## Recurring failures

- Loosening the established winner backfires every time — wider
  probationary residency, delaying promotion to a second hit, and
  over-trusting a single post-eviction reuse all lost hits.
- Spending protected-tier capacity on marginal keys (e.g. ghost hits
  promoted straight to protected) demotes hot residents and costs hits;
  protected residency is the binding constraint.
- Extra probationary dwell time is wasted because reuse on this trace
  arrives fast — the second touch lands inside the narrow probationary
  window.

## Open questions

- Where is the protected/probationary peak? Pushing the ratio further
  toward protected (85/15, 90/10) is untested versus the current setting.
- Reuse-distance / ghost memory was directionally right but mis-placed —
  would routing ghost hits back to *probationary* recover the signal
  without crowding the hot set?
- Are the protection signals additive or redundant (segmentation +
  reference bit / midpoint)? Untested.
- Is the current winner a true local optimum, or have the wrong axes been
  probed? Orthogonal levers (eviction-victim selection, scan resistance)
  remain unexplored.

## Per-policy notes

- **LRU (baseline):** evicts reused keys as readily as one-hit-wonders;
  every reuse-aware variant beat it. Pure-recency baselines are exhausted.
- **SLRU / segmented:** the strongest family and the standing winner; its
  default tuning already sits near-optimal and errs only toward
  promote-early/protect-generously. Tune within it rather than re-deriving
  that it beats LRU.
- **Single-list approximations (midpoint insertion, CLOCK):** also lift
  hit-rate, so two-segment bookkeeping is not strictly required — but
  explicit segmentation has won head-to-head so far.
- **Ghost-list augmentation:** has only hurt when it promotes to protected;
  its value, if any, depends on demoting the promotion target to
  probationary.
