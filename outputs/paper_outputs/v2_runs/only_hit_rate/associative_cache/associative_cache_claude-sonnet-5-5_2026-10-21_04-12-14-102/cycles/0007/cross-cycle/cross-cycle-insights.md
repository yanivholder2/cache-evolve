## Recurring wins

- A structural split of each bucket into probation and protected segments, with first-hit promotion, is the one design that clearly beat plain LRU. Every later gain was small next to it.
- A lean positional split with no extra per-entry state (no ghost, counters or adaptive targets) captures almost all of that benefit. It was rebuilt on the baseline and won more than once.
- A small probation share (about slots/16) is at or near the optimum. Shrinking it further hurt.
- Wider associativity stacks on the split, but only by a tiny increment. Proportional probation sizing held up better than an absolute size.

## Recurring failures

- Weakening hit-path promotion (transposition, midpoint, half-promotion) lost hit rate. The same goes for delaying promotion (two-hit rule).
- Ghost history and ghost-hit promotion added nothing and displaced entries that had earned protection.
- Frequency windows and recency-stamp victim overrides were flat or negative. The segment boundary already encodes the reuse evidence.
- Placement outside the hashed bucket fails the slot-legality invariant.
- Demotion placement and other policy-preserving tweaks tie their parent exactly. Ties count as Winners but say nothing about quality.
- Per-op-time and metadata-size have been missing from the fitness vector in every cycle. No cost claim (speed, footprint, wider scans, reordering on hits) has ever been confirmed or refuted. This is the biggest systemic gap.
- Several variants compare against the baseline instead of the current best parent, so their gains are not increments.

## Open questions

- Can per-op-time and metadata-size be made measurable? Judging the rotate refactor and the wide-bucket cost depends on it.
- Do the hit-rate gains survive once speed and size are priced?
- Has the segmented-LRU design reached its ceiling on this trace? Further gains probably need a different admission or eviction family.
- Do results hold on traces other than the LIRS-style one, or is the tuning trace-specific?
- Are exact ties real neutrality or edits that never took effect, given that authors could not inspect the code?

## Per-policy notes

- Plain move-to-back LRU: the baseline. It is beaten by the split but is hard to beat with promotion tweaks alone.
- Segmented LRU (probation/protected): the incumbent. Hit-rate levers are exhausted (size, promotion, demotion, ghost state, associativity). The remaining questions are cost and robustness.
- 2Q-style and ghost-ring variants: adaptive targets and ghost state did not beat the fixed lean split.
- Frequency-aware and stamp-based victim selection: no gain over probation-first eviction.
- Wide buckets (double associativity) with slots/16 probation: the current hit-rate base, but its scan cost is unpriced.
