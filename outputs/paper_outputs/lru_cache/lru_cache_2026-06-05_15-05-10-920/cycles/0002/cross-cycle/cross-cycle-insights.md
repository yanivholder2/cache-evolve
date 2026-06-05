# Cross-Cycle Insights

## Recurring wins

- **Protecting high-frequency items**: Both frequency-based eviction and zone-based partitioning consistently improve hit rate by shielding frequently-accessed items from eviction. This signal dominates pure recency and generalizes across refinement attempts.
- **Multi-signal combination**: Hybrid approaches that combine two axes (frequency + recency, or frequency + temporal adaptation) outperform single-axis strategies, indicating the workload has orthogonal structure worth exploiting.
- **Temporal adaptation through decay**: Periodic frequency reset (halving counters at regular intervals) captures workload phase shifts, allowing the cache to shed ancient hotspots and weight newly-popular items. This improved upon static frequency counters without requiring zone-based partitioning.

## Recurring failures

- **Conservative re-insertion placement**: Ghost-list "second chance" insertion at the middle of the LRU order (Cycle 0001, v003) degraded performance despite correct core intuition. The lesson: protecting valuable items requires higher priority than middle-tier placement.
- **Reversing proven tie-breaking**: Deviating from LRU scan order (Cycle 0002, v001: tail-to-head instead of head-to-tail) hurt performance, showing that the original head-to-tail LRU order correctly captures temporal locality and age-based coldness.

## Open questions

- **Decay generalization**: Does the capacity × 16 interval for periodic frequency halving hold across different workload traces and cache sizes, or is it tuned to this scenario? Can decay thresholds be made adaptive?
- **Comparative performance**: In absolute terms, how does periodic decay (Cycle 0002, v003) compare to zone-based hybrid partitioning (Cycle 0002, v002)? Are gains complementary or redundant?
- **Composability**: Can periodic decay be combined with zone-based partitioning to independently adapt cold and hot zone frequencies? Would this yield further improvement or diminishing returns?
- **Scaling behavior**: How do frequency-based and zone-based strategies scale in memory overhead and eviction cost as capacity grows?

## Per-policy notes

- **Frequency-based eviction**: Baseline improvement from protecting hot items, but static counters leave early hotspots entrenched. Periodic decay resolves this and is now the strongest single-policy approach tested.
- **Zone-based partitioning**: Hybrid variant (frequency for hot zone, LRU for cold zone) is a viable simpler alternative with marginal performance loss. Strong candidate if computational cost or implementation complexity matters.
- **Ghost-list/second-chance**: Conservative middle-tier insertion was fundamentally wrong. Not abandoned yet—could be revisited with aggressive high-priority insertion or combined with frequency tracking, but lower priority than exploring decay and zone-based refinements.
