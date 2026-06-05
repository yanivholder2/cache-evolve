# Cross-Cycle Insights

## Recurring wins

- Eviction *ordering* beats added residency: gains come from choosing
  which resident key dies first while the capacity bound stays fixed.
- A "second access earns protection" signal — whether a protected
  segment, midpoint insertion, or a reference bit — consistently lifts
  hit-rate by separating one-hit-wonders from genuine reuse.

## Recurring failures

- Pure-recency LRU baselines keep losing: treating a reused key like a
  one-hit-wonder is the weakness every successful variant exploits.

## Open questions

- Does explicit segmentation win on margin, robustness, or both versus
  lighter single-list approximations (midpoint, CLOCK)?
- How sensitive is hit-rate to the protected/probationary split and the
  promotion threshold? So far assumed, not tuned.
- Are the protection signals additive (segments + reference bit) or
  redundant when combined?

## Per-policy notes

- SLRU (two-segment): highest-scoring mechanism so far; carries explicit
  bookkeeping cost whose cost/benefit edge over cheaper signals is
  unconfirmed.
- Midpoint insertion / CLOCK reference bit: cheaper single-list
  approximations that also beat LRU, suggesting the gain does not strictly
  require two-segment structure.
- Pure LRU: the recurring loser on reuse-heavy traces; drop as a baseline.
