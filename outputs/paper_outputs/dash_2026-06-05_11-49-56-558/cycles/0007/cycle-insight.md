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
