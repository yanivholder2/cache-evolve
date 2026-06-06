# Cycle 0009 Insight

## Cohort outcome

The cohort tested three signal-blending hypotheses against the IRR baseline: reintroducing frequency counting as the primary eviction signal with IRR as a tie-breaker (built on the older frequency baseline), adding a coarse visit-count tie-breaker to pure IRR, and replacing raw IRR with a staleness-weighted product. The winner (v001) advances despite regressing from its own parent — confirming that the frequency-counting baseline carries a meaningfully higher absolute score than the IRR baseline, enough to survive the removal of ghost admission.

## Patterns

- Frequency counting as the primary eviction signal outperforms pure IRR in absolute hit-rate terms on this workload, even after ghost admission is stripped out. The cycle-7 and cycle-8 work on the IRR baseline produced real relative gains from a lower starting point, but the IRR baseline never caught the frequency baseline. The search spent two cycles optimizing from a weaker foundation.
- Ghost admission was not an exhausted direction — it was actively contributing to the frequency-baseline design. Removing it regressed v001, but v001 still won because the frequency signal's advantage over pure IRR is large enough to absorb the loss. Ghost admission remains a positive mechanism within the frequency design specifically; the prior conclusion that it was exhausted applied to the ghost-ghost-with-IRR combination and should not have been generalized.
- IRR-based tie-breaking — whether as a secondary tie-breaker over equal-frequency entries (v001) or as a visit-count secondary over equal-IRR entries (v002) — adds no measurable lift. Equal-candidate collisions under the dominant signal are rare enough on this LIRS-structured trace that secondary tie-breakers never activate consequentially. Both mechanisms are inert.
- Staleness-weighted IRR (v003) produces a genuine small gain on the IRR baseline, confirming that silent tight-burst entries are a real class of misclassification under pure IRR. The gain is modest because the frequency of that pattern is low, but the direction is validated.

## Open questions

- Does restoring ghost admission to v001 (frequency primary + IRR tie-breaker + ghost) recover the regression and yield a combined gain, or does the IRR tie-breaker interact badly with ghost-shortcut promotions?
- Is the staleness-weighting improvement (v003) additive with the frequency-primary signal — i.e., does a staleness-weighted frequency score, or a staleness-weighted IRR tie-breaker, improve on v001?
- Given that two cycles of IRR-baseline work peaked below the frequency baseline, is the IRR signal fundamentally weaker on this workload, or did it simply need a different structural context (e.g., combined with ghost admission)?

## Continuation

Next cycle should use v001 as the new baseline (frequency primary, IRR tie-breaker, no ghost, 80/20 split, LRU-quarter and LRU-half scans). The highest-priority thread is restoring ghost admission to v001 to test whether the regression is recoverable — ghost admission was a net-positive mechanism in the frequency design and was only dropped based on a conclusion derived from the IRR baseline where it is genuinely non-complementary. Second priority is applying the staleness-weighting idea (validated in v003) to the frequency-primary signal. Secondary tie-breakers (IRR-on-frequency-ties, visits-on-IRR-ties) are both inert and should not be revisited.
