# Cross-cycle insights

_Only one cycle (0001) is in the ledger so far. These are first signals,
not yet confirmed trends; treat as provisional until a second cycle
either reinforces or contradicts them._

## Recurring wins

- Structural promotion-path changes (replacing a whole-bucket shift with
  an O(1) tail insert) have produced the only Pareto-front landing so
  far, and on the latency axis. Cheap, mechanical fast-path edits look
  like the most reliable lever to date.

## Recurring failures

- Pure-recency retention bets (biasing promotion toward the MRU end)
  have not paid off: on the scan/loop-heavy trace this keeps soon-cold
  keys resident and costs hits. Recency appears to be the wrong signal
  for this workload.
- Single-lever, near-independent cohorts probe which axis is cheapest to
  move but do not coordinate toward a target; so far the axis that moved
  was not the one the bet aimed at.

## Open questions

- Can the O(1) tail-insert latency win be decoupled from the recency
  policy geometry it rode in with, keeping the speed without the
  hit-rate penalty?
- Do frequency- or reuse-distance-aware retention signals outperform
  recency on this trace, as the workload character suggests?
- Were the latency-reorder and metadata-shrink levers dominated,
  behavior-neutral, or simply unrealized? Worth retrying as clean,
  behavior-preserving changes.

## Per-policy notes

- Recency-favoring (LRU-style) promotion: actively penalized on the
  scan/loop-heavy, LIRS-style trace; bias toward MRU retains soon-cold
  keys. Avoid further pure-recency promotion bets here.
- The latency gain remains coupled to one policy's bucket geometry
  rather than being a policy-agnostic change; isolate before
  generalizing.
- No frequency- or reuse-distance-based policy has been exercised yet —
  an untested direction the workload appears to reward.
