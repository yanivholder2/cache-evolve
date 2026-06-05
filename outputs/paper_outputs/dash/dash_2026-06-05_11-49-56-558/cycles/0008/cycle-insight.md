# Cycle 0008 Insight

## Cohort outcome

This cycle probed three extensions of the Cycle 0007 champion (frequency-guard eviction + second-chance stash recovery): a geometry shift toward a larger normal tier, a change to second-chance insertion position, and a stricter promotion gate. Only v001 — widening the normal/stash split from 3/4 to 7/8 — won, delivering a modest positive lift; the others were neutral-to-harmful or only marginally useful.

## Patterns

- **Normal-tier geometry is still a live lever, but returns are shrinking.** Expanding the normal tier to 7/8 of total capacity continued the upward trend established when the split moved from equal halves to 3/4, yet the absolute gain is meaningfully smaller. The hot set is being captured more completely with each increment, but the marginal value of each additional normal-bucket slot is declining.
- **Second-chance insertion position is a quality gate, not a placement detail.** Inserting stash-evicted items at the MRU end rather than the LRU end of their target bucket caused a clear regression: it granted unproven items immediate promotion candidacy, displacing residents that had earned their position through repeated access. Front (LRU) insertion correctly treats second-chance items as cold probationers, preserving the pipeline's eviction discipline.
- **A stricter single-to-multi-access promotion gate is real but weak.** Requiring two in-target hits before probing promotion filters out a small class of lucky single-hit items, but the workload's hot set has mostly accumulated multi-access evidence before reaching the promotion boundary anyway — leaving little room for this gate to act.

## Open questions

- Is there a natural inflection point in the normal/stash split beyond which the stash becomes too shallow to buffer burst traffic, and the marginal loss from reduced stash headroom overtakes the gain from wider normal capacity?
- Could v001's geometry shift and v003's stricter promotion gate compound, given that they target distinct sites (capacity allocation vs. promotion selectivity)?
- At what split ratio does the frequency-guard eviction mechanism start to lose effectiveness, since a larger normal tier means fewer eviction decisions per unit time relative to the working set?

## Continuation

Geometry (normal-tier width) is now the clearest remaining lever: the next hypotheses-writer should test whether a further split increase (e.g., 15/16 or near-elimination of the probing tier) continues to help or crosses the point where stash shallowness becomes a visible loss source. A targeted combination of v001's geometry with v003's stricter promotion gate is worth one test, as the mechanisms are independent in principle. The Back-insertion path for second-chance recovery is definitively contraindicated and should not be revisited. If geometry fully diminishes, the remaining open axis from prior cycles — soft-capping `lfu_counter` to prevent staleness — becomes the next candidate.
