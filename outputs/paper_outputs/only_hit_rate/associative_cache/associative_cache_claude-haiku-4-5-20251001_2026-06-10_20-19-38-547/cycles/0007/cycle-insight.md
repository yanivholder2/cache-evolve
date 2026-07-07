# Cycle 0007 Insight

## Cohort outcome

This cycle tested three refinements to the two-tier stash design (inherited from Cycle 0006): asymmetric partition tuning, access-count-based promotion filtering, and phase-aware tier rebalancing. Only v001—an asymmetric 18% hot / 7% cold split—achieved hit-rate gains. The winner confirms that the two-tier mechanism is resilient and capable of further optimization through resource allocation, while filtering and counter-manipulation strategies remain unproductive.

## Patterns

- **Partition asymmetry unlocks skewed re-access patterns**: v001's success by shifting capacity from cold (12.5% → 7%) to hot (12.5% → 18%) reveals that the workload's evicted items exhibit predictable, asymmetric re-use: most evictions land in cold initially, but a concentrated subset re-access repeatedly. By allocating more capacity to the hot tier (where re-accessed items live), the design reduces overflow pressure on these valuable items. This refines Cycle 0006's insight that stash *organization* matters; it now extends to recognizing that tier *proportionality* should reflect workload re-access distribution.

- **Filtering at promotion fails as filtering at insertion did**: v002's three-tier access-count filtering gated promotion to prevent hot-tier pollution, but the workload has no pollution problem—it benefits from eager promotion. This mirrors Cycle 0005–0006's failed collision-aware and occupancy-aware filtering at insertion; the pattern is consistent: the workload does not require filtering. It requires capacity and eager promotion decisions. Delaying promotion (v002) trades hit-rate for reduced hot-tier churn, which is the wrong trade-off.

- **Counter manipulation remains disconnected from phase adaptation**: v003's counter-boost (amplifying hot-item counters during phase transitions) performs identically to Cycle 0006's counter-reset strategy. This confirms that the problem is not the *mechanism* of counter adjustment but the *axis*: item-level counter manipulation does not translate into effective eviction policy during workload phase transitions. Phase detection works; phase *response* via counters does not.

- **The two-tier promotion-on-hit design is proven**: Cycles 0006–0007 show the core mechanism (cold → hot on re-access) is sound. Refinements occur at the partition level, not the mechanism level. This suggests the design space for stash improvement now lies in partition tuning, organizational strategies orthogonal to promotion, or coupling phase detection with non-counter-based responses (e.g., dynamic tier rebalancing).

## Open questions

- How finely should partition ratios be tuned? Is the 18/7 split near-optimal for this workload, or do narrower adjustments (e.g., 16–20% hot, 5–9% cold) yield incremental gains?
- Are there organizational strategies beyond asymmetric partitioning that better exploit skewed re-access patterns—e.g., frequency-based tiers, time-windowed recency, or multi-level hierarchies?
- Can phase detection be coupled with dynamic partition rebalancing (not counter manipulation) to adapt tier allocation during transitions, and would such coupling outperform static asymmetric partitioning?

## Continuation

v001's success pivots the search space from *filtering logic* (what to reject) to *capacity allocation* (how to distribute resources to match workload characteristics). The asymmetric partition validates that tier proportionality matters; the workload's skewed re-access patterns are now addressable via resource alignment rather than filtering or adaptive counters. v002 and v003 confirm that access-count filtering and counter-level phase responses are unproductive. Next cycle should pursue one of two paths: (1) **Refine partition tuning**: test the 18/7 split against nearby ratios (e.g., 15–20% hot, 5–10% cold) to locate the local optimum and confirm whether marginal gains remain within partition space, or (2) **Explore organizational alternatives**: evaluate frequency-based tiering, multi-level stash hierarchies, or predictive prefetching to hot tier that exploit workload structure without filtering. If partition refinement yields only marginal gains and phase-detection coupling (via dynamic rebalancing, not counters) remains untested, defer it as a future escape route pending confirmation that static allocation has saturated.
