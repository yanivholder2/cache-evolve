## Recurring wins

- A structural probation/protected split beats plain LRU. The gain comes from having the split, not from tuning it.
- A lean positional split (no ghost, counter or adaptive state) captures nearly all the benefit. It has been rebuilt independently several times with the same result.
- A small probation segment (about slots/16) is at or near the optimum. Tuning the size gave small gains and then turned negative.
- Wider associativity stacks on the split, but the increments shrink with each doubling (clear, then marginal, then tiny). Misses now look capacity- or recency-driven, not conflict-driven.
- Speed-motivated refactors (rotate rewrite, skip-move) tie their parents on hit rate. They count as Winners by default, and their goal is unjudged.

## Recurring failures

- Weakening or delaying promotion (transposition, midpoint, two-hit rule, ghost-hit promotion) loses hit rate. New keys need first-hit promotion and enough residency to earn it.
- Insertion-position changes (victim-end/LIP, mid-probation) lose hits at both extremes.
- Eviction overrides lose hits: small-window frequency victim choice (flat), recency-stamp compare, and one-bit reuse flag. The segment boundary already encodes the reuse evidence.
- Ghost history and adaptive targets add state without gain.
- Larger probation fractions (slots/8) take protected capacity and gain nothing. Shrinking below slots/16 also hurts.
- Placement outside the hashed bucket (two-choice) fails the slot-legality invariant.
- Demotion placement is exactly neutral.
- Per-op-time and metadata-size have been unmeasured in every cycle. No cost-side claim has ever been confirmed or refuted.
- Several variants compare against the baseline rather than the incumbent, so their gains can't be read as increments.
- Authors can't read the overlay code. An exact tie may mean the edit had no effect.

## Open questions

- Do per-op-time and metadata-size fit in the fitness vector? Until they do, nothing about wide buckets (up to 8x), skip-move or the rotate refactor can be priced.
- Does the marginal hit-rate gain from very wide buckets survive the cost of longer scans?
- Is the ceiling for this design on this trace reached? Would a different policy family, such as frequency-aware admission or a separate admission filter, add anything?
- Do the results hold on traces other than the LIRS-style one, given the tuning may be trace-specific?
- Would a multi-bit or recency-aware reuse signal work where the one-bit flag failed?
- Are exact hit-rate ties real neutrality, or are the edits ineffective?

## Per-policy notes

- Plain move-to-back LRU: the baseline. Weakened-promotion variants (2Q-style half-promotion, transposition) are worse than it at small associativity.
- Segmented LRU (probation + protected, first-hit promotion, slots/16 probation, positional): the current base and the main source of gains. Tuning knobs are essentially exhausted.
- Wide-bucket segmented LRU: the best hit rate so far, with shrinking returns per doubling. Its speed cost is unknown.
- Skip-move / rotate variants: hit-rate neutral and unjudged on speed.
- Frequency, ghost and reuse-bit variants: flat or negative on top of the split.
- Next steps: price the existing designs first, then try a different policy family or other workloads. Avoid more split knobs.
