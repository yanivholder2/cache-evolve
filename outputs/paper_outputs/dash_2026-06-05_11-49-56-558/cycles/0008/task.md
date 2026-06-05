# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0008

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants probed insertion-position ordering — ghost re-admission positioning, a canonical promotion/demotion asymmetry in the normal tier, and a probationary zone in probing — and all three regressed relative to the parent. v001 won by being the least harmful: ghost re-admission at MRU produced only marginal noise, while the other two incurred clear regressions.

## Patterns

- **MRU positioning dominates universally on workload-a.** Every attempt to withhold MRU status — whether at the ghost re-admission boundary, the demotion cascade, or the probing entry point — hurt hit-rate. The workload's reuse gaps are short enough that displaced or newly-promoted items reliably get re-accessed before eviction, making probationary or LRU-entry positioning consistently punishing rather than protective.
- **The ghost set is now exhausted as a lever in all configurations.** Back-insert on re-admission moved the needle marginally in the wrong direction, confirming the earlier finding that ghost re-admission delivers noise-level movement regardless of position. The stash eviction boundary simply is not where meaningful hit-rate is lost on this workload.
- **Textbook segmented-LRU asymmetry (promotions at MRU, demotions at LRU) is contraindicated here.** The standard design assumes a clean hot/cold divide, but workload-a's access distribution is warm throughout — items displaced by a hot promotion still have residual heat, and inheriting MRU position in the destination tier is what preserves their next hit. Removing that buffer accelerates churn rather than improving selectivity.

## Open questions

- With MRU universally dominant, is there any meaningful insertion-ordering axis left to probe, or has this design axis been fully exhausted?
- Would workload-a behave differently under a capacity stress test (smaller total cache size), where probationary filtering might finally pay off because good eviction decisions matter more at the margin?
- Is there a genuinely new structural mechanism — adaptive segment sizing, size-aware eviction, or TTL-based decay — that workload-a's stable hot set would respond to?

## Continuation

The ghost set should be permanently retired; it has been shown noise-level across multiple position and size configurations. The universal MRU finding should be encoded as a constraint: any future variant that inserts at LRU/Front for newly-arriving or re-admitted items is likely to regress on workload-a. The search space under the current design appears close to exhausted; the next hypotheses-writer should either introduce a structurally new mechanism (adaptive sizing, eviction cost weighting) or switch to a second workload trace where cold-item churn or scan traffic would make probationary and frequency-aware schemes genuinely competitive.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
