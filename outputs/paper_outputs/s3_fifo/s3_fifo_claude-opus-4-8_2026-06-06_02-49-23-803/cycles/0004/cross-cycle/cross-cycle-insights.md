# Cross-Cycle Insights

## Recurring wins

- **Capacity reallocation (small/main split)** is the most reliable
  hit-rate lever: every cycle that shifted capacity from the admission
  filter into main raised hit-rate. The small queue only needs to be
  wide enough to screen one-hit-wonders; freed slots are worth more as
  main residency.
- **Rewarding demonstrated reuse** (a taller frequency-saturation ceiling
  for already-hot keys) gives small but real gains, and stays orthogonal
  to the split — durability-for-proven-keys is independent of how-many-fit.
- **Tuning ghost length to ~50% of main** is the standout: it banks a
  put-latency win and holds (even slightly exceeds) hit-rate — a rare
  non-tradeoff that strictly dominated its parent. Near-reuse worth
  keeping lives in the 25–50% band of recent evictions.
- **Metadata discipline**: every winning change retuned an existing field
  or constant. Retuning beats adding struct state, which costs metadata
  for no benefit.

## Recurring failures

- **Loosening retention** (promote on first reuse, re-admit ghosts at
  freq>0) consistently lowers hit-rate. The baseline's strict
  two-touch/freq=0 filtering is load-bearing noise suppression, not
  over-aggression.
- **Widening the ghost queue** repeatedly regresses hit-rate: this trace
  rewards near reuse, so longer reuse-distance detection readmits stale
  far keys that displace recently-reused entries — and it grows the struct.
- **Bounding the second-chance rotation loop** buys no tail-latency win
  and sheds reuse-proven keys: the put tail here is driven by the ghost
  membership scan, not resident rotation.
- **Reading per-op-time deltas as signal** when no scan length changed:
  these are run-to-run jitter unless a change actually alters ghost or
  resident traversal length.

## Open questions

- Where is the split knee — does the next narrowing finally under-screen
  one-hit-wonders and regress, or does the flattening monotone continue?
- Within the 25–50% ghost band, which exact point maximizes the latency
  win while preserving the hit-rate gain?
- With the split × freq-ceiling grid now saturated, is the workload
  hit-rate-bound at this capacity, or is any hit-rate lever left?
- Are there other linear scans (beyond the ghost membership scan) whose
  length could be trimmed for latency without touching hit-rate?

## Per-policy notes

- **S3-FIFO (workload-a, all cycles):** strict admission/promotion
  thresholds are protective; tune capacity allocation, never loosen the
  filter. Reuse rewarded is narrow but extends past the most-recent 25%
  of evictions — peaking around a 50%-of-main ghost window. Latency wins
  come from the ghost membership scan, not the resident rotation. The
  hit-rate grid (split × freq-ceiling) is essentially saturated; the live
  axis is the latency/hit-rate ghost frontier.
