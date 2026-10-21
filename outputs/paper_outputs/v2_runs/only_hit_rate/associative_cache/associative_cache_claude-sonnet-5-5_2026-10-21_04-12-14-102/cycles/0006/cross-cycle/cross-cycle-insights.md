## Recurring wins

- Structural separation of one-touch from re-referenced entries (a probationary/protected split inside each bucket) is the one lever that clearly paid off. It beat both geometry and promotion tweaks.
- A small probation segment (about slots/16, roughly 4 slots) won from several independent routes. The size trend has turned, so this is at or near the optimum.
- A lean positional split with no per-entry state (no ghost, counters or adaptive targets) captures almost all the gain over plain LRU. It was rebuilt on the baseline more than once.
- Wider associativity stacks with the small-probation split. This is only shown against the baseline, not as an increment over the best parent.
- First-hit promotion with full move-to-back recency is hard to beat.

## Recurring failures

- Weakening or delaying promotion (transposition, midpoint, half-promotion, two-hit rule) lost hit rate. Short buckets and small probation leave no room to wait for a second hit.
- Ghost history and ghost-hit promotion added nothing. A ghost hit is weak reuse evidence here and can displace entries that had earned protection.
- Small-window frequency victim selection and recency-stamp overrides of probation-first eviction were flat or negative. The segment boundary already encodes the reuse signal.
- Placement outside the hashed bucket (two-choice) fails the slot-legality invariant, even on the baseline.
- Probation-size shrinking below the sweet spot, and demotion-placement tweaks, are flat or slightly worse. Both are exhausted.
- Per-op-time and metadata-size have never been measured, so no cost-side claim has been confirmed or refuted. Several "wins" are exact ties, and some are comparisons to the baseline rather than the parent.

## Open questions

- Can per-op-time and metadata-size be added to the fitness vector? This blocks judging the rotate rewrite, wider buckets and the "lean" claims.
- Does the wider-associativity gain hold as an increment on the current best parent once scan and shuffle cost is counted?
- Are the exact-tie variants truly neutral, or are they edits that never changed behaviour, such as a demotion path that rarely fires?
- Do the results hold on traces other than the LIRS-style one, or is the tuning specific to it?
- Is there any headroom left in protected-segment handling, or is the design at its ceiling for this trace?
- Could a stricter ghost rule (a ghost hit plus a second signal) rescue ghost history? This is low priority.

## Per-policy notes

- Plain move-to-back LRU: the baseline. Every policy tweak that weakened promotion lost to it. Wider buckets gave only a small gain, so hash imbalance is a minor miss source.
- Segmented LRU (probation + protected): the winning family, and policy rather than geometry limits hit rate. Lean positional version with a small fixed probation is the reference base. Further levers inside it (demotion placement, promotion rules, size, stamps) are exhausted.
- Ghost-ring and adaptive 2Q-style variants: no gain over the fixed split, and they add unpriced state. Do not pursue without a stricter promotion rule.
- Frequency-aware and two-choice variants: frequency windows were neutral, and two-choice was rejected at the invariant gate.
