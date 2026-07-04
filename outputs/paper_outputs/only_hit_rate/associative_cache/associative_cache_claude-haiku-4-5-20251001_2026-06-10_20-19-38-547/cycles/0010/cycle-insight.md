# Cycle 0010 Insight

## Cohort outcome

This cycle tested three hypotheses against the converged two-tier design (20% hot / 5% cold baseline from Cycle 0009): cold-tier LRU eviction, three-level tiering with intermediate warm tier, and frequency-weighted promotion scoring. Only v003—frequency-weighted promotion—delivered breakthrough gains (+5.0% relative hit-rate improvement), revealing that the workload's re-access distribution is highly skewed and that promotion *prioritization* is the critical lever, not eviction order within tiers.

## Patterns

- **Promotion prioritization unlocks gains where eviction order does not**: v001's null result (cold-tier LRU providing no improvement) contradicts the hypothesis that temporal locality principles (proven modestly in hot-tier LRU, +0.5% in Cycle 0009) generalize across tiers. v003's +5.0% gain demonstrates the real axis: cold-tier items do not need better eviction ordering; they need *selective prioritization* when space in hot tier is limited. The frequency-weighted scoring (`cold_access_count / (1.0 + time_since_eviction_ms)`) identifies which evicted items are most likely to be re-accessed, then allocates scarce hot-tier capacity to those high-value recoveries first. This shifts the design paradigm from "order evictions well within tier" to "prioritize promotions by workload value."

- **Workload's Zipfian characteristics favor concentrated hot-tier allocation**: v003's mechanism explicitly rewards items showing multiple re-accesses in cold tier and recent eviction history. The +5.0% gain validates that a small fraction of the cache's evicted items (those with high promotion scores) should monopolize hot-tier capacity rather than sharing it equally. This is fundamentally different from the partition-tuning axis (Cycles 0007–0009), which optimized *total* capacity allocation; v003 optimizes *per-item* allocation within that capacity based on eviction-time behavior.

- **Two-tier promotion-on-hit plus prioritization is more powerful than multi-tier structure**: v002 (three-tier with intermediate warm buffer) was never tested due to build failure, but v003's success on the two-tier baseline suggests that adding architectural complexity (extra tier) may not be necessary if promotion decisions are intelligent. The frequency-weighted score captures the intent of a warm tier—buffering items that oscillate between frequent and infrequent access—without the implementation overhead of tracking three separate stash vectors.

- **Cycles 0006–0010 form a convergence arc around adaptive promotion**: The design has evolved from core mechanism (two-tier, Cycles 0006–0008) to partition allocation (Cycles 0007–0009) to adaptive prioritization (Cycle 0010). Each phase yielded diminishing returns until a new axis was unlocked. v003's breakthrough suggests the design has found a new equilibrium where *which* items get promoted matters more than how many or how they are evicted.

## Open questions

- Does frequency-weighted promotion interact additively with hot-tier LRU (from Cycle 0009, +0.5% gain)? Could combining both mechanisms achieve cumulative gains beyond v003's +5.0%?
- How sensitive is v003's gain to the promotion-score formula? Alternative weightings (e.g., emphasizing frequency over recency, or using different time-decay functions) might yield further refinement.
- Can v002 (three-tier) be salvaged and tested with frequency-weighted promotion applied to both warm→hot and cold→warm transitions? Or does the two-tier + frequency-weighted design already capture that value?

## Continuation

v003's +5.0% breakthrough pivots the search space from *capacity allocation and eviction order* to *promotion prioritization based on eviction-time behavior*. The frequency-weighted score proves that items showing concentrated re-access patterns while in cold tier are high-value recoveries and should claim hot-tier space preferentially. v001 and v002's null/build-failure results confirm that extending eviction-order principles or adding architectural tiers without intelligent prioritization yields no gains. Next cycle should pursue one of two paths: (1) **Refine the promotion-scoring mechanism**: test alternative frequency-vs-recency weightings, time-decay functions, or formula variants to locate a local optimum within the frequency-weighted framework, and verify whether combining with hot-tier LRU achieves additive gains, or (2) **Extend frequency-weighted prioritization to multi-tier designs**: fix and test v002 (three-tier) with frequency-weighted promotion at both warm→hot and cold→warm transitions to see whether an intermediate tier combined with prioritization outperforms two-tier + frequency-scoring. If formula refinement yields only marginal gains beyond +5.0%, accept v003 as the next stable design point and document the shift from partition-driven to prioritization-driven optimization.
