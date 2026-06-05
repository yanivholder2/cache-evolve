# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0009

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants probed distinct new axes — eviction selectivity within a tier, unconditional stash-eviction recovery, and timestamp-driven promotion — and all produced some improvement over their respective parents. v001, which guards confirmed-reuse residents from eviction by zero-access newcomers, won clearly; the second-chance and timestamp variants improved but did not match it.

## Patterns

- **Eviction selectivity is a live lever even after topology and insertion-position are exhausted.** Preferring to evict items that have never been accessed within their current tier over items with at least one confirmed hit produces a clear gain. The probing boundary turns out to be a meaningful loss source: newly promoted items with no in-tier reuse had been displacing established residents on workload-a's working set.
- **Second-chance promotion definitively outperforms the ghost set.** Unconditionally moving every stash-evicted item to the LRU end of its target bucket works better than the ghost set's conditional, event-driven re-admission — confirming that the ghost set's weakness was its dependency on a future external re-insertion event, not the insertion position chosen. The second-chance mechanism's gain is real but smaller than the frequency-guard gain, suggesting eviction selectivity was the bigger bottleneck.
- **TimestampLRU replicates established gains without surpassing them.** Decoupling promotion candidacy from insertion order (so a freshly stash-promoted item does not automatically become the next promotee) provides benefit under the Zipfian distribution, but this effect is already partially captured by the confirmed-topology and MRU-insertion combination from earlier cycles.

## Open questions

- Can v001's frequency guard be stacked with v002's second-chance recovery for a compounded gain, since they address different loss sites (probing eviction selectivity vs. stash overflow recovery)?
- Is the zero-access threshold in the frequency guard the right cutoff, or would a slightly higher threshold (e.g., evict items with fewer than two in-tier hits) yield further improvement?
- Does the frequency counter reset on stash demotion interact with the second-chance mechanism in a useful or harmful way if both are active simultaneously?

## Continuation

v001's frequency guard is the new champion and the most significant improvement in several cycles. The next hypotheses-writer should prioritise stacking v001 with v002's second-chance mechanism to test whether the two gains are additive — they target different parts of the pipeline (probing eviction vs. stash overflow). If that compound wins, a further probe of the frequency threshold (counts > 0 vs. counts > 1) is a natural follow-on. TimestampLRU (v003) need not be abandoned but should be tested as an addition to the winning frequency-guard baseline rather than as a standalone replacement.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

This cycle tested three extensions of Cycle 0006's frequency-guard winner: stacking it with second-chance stash recovery, grafting timestamp-based promotion onto it, and raising its eviction threshold. Only v001 — the direct stacking of both prior-cycle gains — won clearly; the other two variants regressed.

## Patterns

- **The two Cycle 0006 gains are additive.** Second-chance stash overflow recovery stacks cleanly with the zero-access eviction guard because the two mechanisms target genuinely distinct pipeline stages: one governs which resident survives eviction within a normal/probing bucket; the other governs what happens to an item expelled from the stash. Combining them closed both loss sites simultaneously without interference.
- **Timestamp promotion is not compatible with frequency-guard eviction.** Selecting the promotion candidate by maximum `last_accessed` rather than by LRU position allows cold items whose creation timestamp is newer than a hot item's last-access timestamp to win promotion. This cold-item promotion leak more than cancels the eviction-guard improvement — the two axes interact destructively, not independently.
- **The zero-access threshold is exactly right for workload-a.** Raising the guard to also evict single-touch items treats them as cold noise, but on this Zipfian workload a single in-tier access is a reliable reuse predictor. Items with one hit belong to the warm working set and should be protected; tightening the threshold shrinks the protected set too aggressively.

## Open questions

- With probing-eviction selectivity and stash-overflow recovery both active, what loss site remains to be closed — geometry (stash capacity), counter staleness, or something else?
- Could timestamp-based promotion be rehabilitated if creation-time contamination were eliminated — e.g., by only recording a timestamp on the first *hit* rather than on insertion?
- Does the `lfu_counter` reset on stash demotion interact beneficially or harmfully with second-chance recovery, given that re-promoted items re-enter the normal tier with a zeroed counter?

## Continuation

v001 — frequency-guard eviction plus unconditional second-chance stash recovery — is the new champion and confirms the stacking hypothesis explicitly recommended by Cycle 0006. The next hypotheses-writer should treat both mechanisms as a fixed baseline and probe remaining loss sites: geometry adjustment (varying stash-to-normal capacity ratio), or a conservative investigation of whether any counter-management policy (e.g., a soft cap on `lfu_counter` to prevent staleness) yields further improvement. The timestamp promotion axis should be deprioritised unless a variant that avoids creation-time contamination can be precisely specified.


### Cycle 0008

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
