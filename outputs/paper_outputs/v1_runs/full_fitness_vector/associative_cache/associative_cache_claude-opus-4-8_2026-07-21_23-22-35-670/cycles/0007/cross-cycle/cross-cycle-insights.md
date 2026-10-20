## Recurring wins

- Trimming dead metadata off the eval path is cheap and monotonic: fields
  carried only by off-path policies peel away with zero hit-rate change,
  cycle after cycle, down to the current `Item` floor.
- Parking state off the measured `Item` (in the bucket struct) is free on
  the size metric — this exploit has held every time it was used, letting
  scan-resistance and segment structure ride at the metadata floor.
- The index-preserving swap for parallel reuse/state vectors reliably
  collapses the quadratic p99 tails that key-matched rebuilds inflate,
  while holding hit-rate bit-for-bit.
- Genuine two-segment separation (SLRU-style) is the one lever that broke
  the long-standing hit-rate ceiling, and it does so at the floor.
- Probationary-front admission is a settled optimum: a fresh unproven miss
  being the next eviction is exactly what makes it work.
- Permanent binary reuse-protection fits this trace better than any decaying
  variant.

## Recurring failures

- Frequency/LFU victim selection loses to recency-front every time it is
  tried — reuse recency is a better signal than reuse count here.
- Second-chance / CLOCK-style aging sheds protection too fast and collapses
  toward plain LRU; the trace is not capped by cold-once-hot squatters.
- Widening bucket width / per-set associativity regresses: it lengthens the
  linear scan (per-op tail cost) and coarsens hash spread; set diversity
  beats per-set depth at this size.
- Layering explicit reuse protection onto front-admission is redundant —
  under move-to-back-on-hit the front slot already segregates reused items.
- Midpoint / wider-probation admission is strictly worse than front.
- The protected-cap knob is falsified at both extremes (too-aggressive
  demotes real winners; near-total degenerates the two segments into LRU).

## Open questions

- Is diverse-small-bucket two-segment SLRU the structural hit-rate ceiling
  for this trace, now that more depth, more memory (ghosts), and more
  protection have all been falsified?
- Where does the protected cap actually optimize? Both walls are ruled out;
  only the mid-range is unexplored, and it may turn a hold into a gain.
- Can the per-access protected rotation be shed for latency without the
  saturated-bucket hit-rate leak (e.g. lazy demotion only when fully
  protected)?
- How to structurally prevent single-file overlays from silently dropping
  parent-layout wins (the recurring metadata-by-omission regression)?

## Per-policy notes

- Recency-front / ClassicLRU: the settled baseline eviction signal; every
  attempt to replace or re-time it on a single list has failed.
- Segmented SLRU: current incumbent — the only path to break the ceiling,
  runs at the 16-byte floor; its open cost is per-access rotate/rebalance
  latency, entangled with the saturated (all-protected) bucket edge case.
- LFU / frequency tiebreak: repeatedly regresses; drop as a victim signal.
- 2Q ghost memory: cross-eviction recall over-inflates the protected set and
  erodes reuse-bit discrimination; no hit-rate gain, adds put-side cost.
- CLOCK / second-chance aging: weakens the permanent protection that wins
  here; consistently a loser.
