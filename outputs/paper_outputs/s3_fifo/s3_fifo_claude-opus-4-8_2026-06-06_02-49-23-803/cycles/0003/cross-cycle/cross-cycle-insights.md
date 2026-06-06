# Cross-Cycle Insights

## Recurring wins

- **Capacity reallocation (small/main split)** is the most reliable
  hit-rate lever: every cycle that shifted capacity from the admission
  filter into main raised hit-rate. The gains are monotone but clearly
  flattening as the split narrows.
- **Rewarding demonstrated reuse** (raising the frequency-saturation
  ceiling for already-hot keys) gives small, repeatable hit-rate gains and
  stays orthogonal to the split — durability-for-proven-keys and
  how-many-fit are independent axes.
- **Metadata discipline**: winners consistently retune an existing field
  or constant rather than adding struct state, keeping metadata-size flat.

## Recurring failures

- **Loosening the admission/retention filter** (first-reuse promotion,
  non-zero-freq ghost re-admission) repeatedly lowered hit-rate. The
  baseline's strictness is load-bearing: it screens one-hit-wonders, and
  weakening it lets weakly-touched items crowd out durable entries.
- **Widening the ghost queue** regressed hit-rate and grew the control
  struct — this trace rewards near reuse, not far reuse, so longer
  reuse-distance detection only readmits stale keys.

## Open questions

- Where is the split **knee**? Each narrowing still pays but by less;
  the under-screening crossover sits just past the current narrowest
  split. Stop assuming monotonicity.
- Is there an intermediate ghost length (between ~25% and 100% of main)
  that banks the put-latency win from shrinking without shedding the
  near-reuse hits lost at the shortest setting?
- With latency now scored, do hit-rate and per-op-time pull against each
  other on the same knobs, forcing explicit tradeoffs?

## Per-policy notes

- **S3-FIFO**: small admission queue need only be wide enough to screen
  one-hit-wonders; freed slots are better spent on main residency. Ghost
  length is a latency/hit-rate tradeoff in *both* directions, not a free
  lever — widening hurts hit-rate, shrinking hurts hit-rate but cuts the
  linear membership-scan cost in put. Capacity knobs are not additive
  (stacking a wider ghost on the winning split regressed). Treat
  per-op-time as real only when a change alters a scan length
  (ghost/resident traversal); otherwise it reads as run-to-run noise.
