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
