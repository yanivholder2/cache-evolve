# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0010

## Winners

- v003

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0010/variants/v001/insight.md
- /home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0010/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0010/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0010/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

This cycle probed two further extensions of geometry (pushing the normal/stash split past 7/8 toward near-elimination of the stash) and one counter-accounting fix (crediting a stash-hit access on the `lfu_counter` before target insertion). Both geometry extensions regressed or were neutral; only the counter-credit change won, delivering a modest positive lift.

## Patterns

- **Geometry has reached its effective ceiling at 7/8.** Widening the normal tier further to 15/16 reversed the gain trend: with only two stash slots remaining, cold-arrival bursts that the parent's four-slot stash absorbed without harm now compete directly with hot-tier residents, and second-chance recovery cannot compensate for the lost headroom. The inflection point is clearly between 28 and 30 normal buckets for this workload.
- **The stricter promotion gate is ineffective at the 7/8 geometry.** Requiring two in-tier accesses before promotion targets a loss site — spurious single-hit promotions competing for scarce probing slots — that only exists when the normal tier is narrow. At 7/8 width, items reaching the promotion boundary are already strongly hot; the gate only delays correct promotions without reclaiming meaningful capacity.
- **Pre-insertion counter credit at the stash exit closed a real but modest protection gap.** Without this change, stash-demoted items re-entering the target tier carry a zeroed `lfu_counter`, making them indistinguishable from cold newcomers under `ClassicLRUFreqProtect` and immediately eviction-vulnerable despite having just been accessed. Crediting the access before insertion eliminates that one-access window. The win is genuine but smaller than prior geometry gains, indicating the vulnerability window existed but was not the dominant loss site.

## Open questions

- With geometry exhausted and the stash-exit protection gap now closed, what is the next meaningful loss site — `lfu_counter` staleness for long-lived hot items, or some structural asymmetry in how the probing bucket handles demotions?
- Could a soft cap on `lfu_counter` prevent established hot items from accumulating stale high counts that crowd out newly warm items, and would that interact positively with the stash-exit credit now in the baseline?
- Does the stash-exit counter credit compound with any remaining counter-management axes, or does it represent the last per-item accounting gain available at this geometry?

## Continuation

The 7/8 geometry is confirmed as the ceiling; the next hypotheses-writer should treat it as fixed and not revisit geometry expansion. v003's stash-exit counter credit is the new champion and should be the baseline going forward. The most promising remaining axis, flagged by multiple prior cycles, is soft-capping `lfu_counter` to prevent staleness in long-lived hot items — this is the primary thread to pursue next. The stricter promotion gate (two-hit requirement) is effectively exhausted at this tier width and should be deprioritised unless paired with a geometry change that makes probing slots scarce again.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
