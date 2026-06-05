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
