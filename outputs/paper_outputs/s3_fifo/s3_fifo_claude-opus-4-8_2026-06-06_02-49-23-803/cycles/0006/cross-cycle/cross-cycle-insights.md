# Cross-Cycle Insights

Scope: all completed cycles tune a baseline S3-FIFO cache on workload-a.
Each cycle branches off the prior incumbent. The arc has moved from
hit-rate tuning → a latency/hit-rate frontier → pure structural latency
wins, as hit-rate saturated.

## Recurring wins

- **Capacity reallocation, not per-entry protection.** Repeatedly,
  shifting capacity out of the small admission filter into main raised
  hit-rate. Giving main more room beats giving individual entries more
  second chances.
- **Reward demonstrated reuse only.** Deepening durability for
  already-hot keys (higher freq ceiling) gave small, real, orthogonal
  gains because it never loosens the filter for weakly-touched items.
- **Find the knee, don't assume monotonicity.** Every productive lever
  (split, ghost length) flattened then hit a clear knee — the ghost at
  ~50% of main was the standout, strictly dominating its parent on both
  latency and hit-rate.
- **Structural latency fixes over length trims.** Once tuning saturated,
  eliminating scans structurally (O(1) ghost membership index,
  scanning main before small) moved long-stuck latency floors.
- **Metadata discipline.** Winners retuned existing fields/constants
  rather than adding scored struct state; the scored Entry stayed 24B.

## Recurring failures

- **Loosening the filter / retention.** First-reuse promotion, non-zero
  ghost re-admission, and freq=1 re-admission seeds all regressed —
  the baseline's strict freq=0 re-proving is load-bearing, filtering
  short-lived one-hit-wonders.
- **Widening the ghost queue.** Longer reuse-distance coverage is never
  rewarded on this trace (it readmits stale far-reuse keys) and adds
  metadata; over-trimming the ghost also sheds legitimate near reuse.
- **Curing absent pathologies.** Bounding the resident rotation loop
  bought no tail win — the put tail is driven by the ghost membership
  scan, not resident rotation.
- **Stacking saturated levers.** The split × freq-ceiling grid compounded
  only at the margin once both were near-best; no new regime unlocked.

## Open questions

- With hit-rate and the two main scans addressed, is workload-a now at
  both a hit-rate and a median-latency floor at this capacity?
- Do the median-latency wins survive a p99 reckoning? Median structural
  wins keep redistributing cost into the tail (hash-resize spikes,
  reordered rare paths).
- Is there an O(1) membership structure without the parallel hash set's
  tail spikes (e.g. reusing the resident index)?
- Is the dual-source (main-victim) ghost content net-positive once its
  latency churn is weighed against its thin hit-rate gain?

## Per-policy notes

- **S3-FIFO (workload-a):** Strict promotion/ghost thresholds are
  protective noise filters — tune capacity around them, never weaken
  them. This trace rewards *narrow* reuse distance: the ghost's sweet
  spot is ~50% of main; both wider and much narrower hurt. Ghost
  *content* (admitting aged main victims) is a thinner lever than length
  and is nearly tapped. Latency tracks specific scans (ghost membership
  on put; small-vs-main order on get), each better solved structurally
  than by trimming. Per-op-time deltas are noise unless a change alters
  an actual scan length; unscored container growth is now a live cost.
