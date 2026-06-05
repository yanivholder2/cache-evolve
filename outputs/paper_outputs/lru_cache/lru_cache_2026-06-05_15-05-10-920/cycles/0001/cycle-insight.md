# Cycle 0001 Insight

## Cohort outcome

This cycle tested three strategies for improving hit rate by recognizing non-uniform access patterns: frequency-based eviction (v001, winner), zone-based partitioning (v002), and ghost-list "second chance" handling (v003). The winning frequency-based approach and the zone-partitioning strategy both significantly improved hit rate by protecting frequently-accessed items, while the ghost-list variant degraded performance through conservative re-insertion placement.

## Patterns

- Both high-performing variants (v001 and v002) converged on the same insight: the workload exhibits access-skewed patterns where some items are accessed far more frequently than others, and protecting these "hot" items from eviction is more important than pure recency.
- Frequency-based eviction (global counters) and zone-based partitioning (local promotion thresholds) achieved comparable gains (~9% and 8.3% respectively), suggesting multiple valid approaches to the same problem.
- The ghost-list strategy failed not because the core intuition was wrong, but because placing re-inserted items at the middle of the LRU order was too conservative—they should have been prioritized higher to match the observed benefit of protecting valuable items.

## Open questions

- Why does the frequency-based approach narrowly outperform zone-based partitioning on this workload? Is there a secondary effect (e.g., better cache locality, lower tie-breaking variance) that favors global counters?
- Could the ghost-list mechanism be salvaged by adjusting the insertion position closer to the tail, or by coupling it with frequency tracking?
- How do these strategies scale in terms of eviction cost and memory overhead as capacity grows?

## Continuation

Pursue refinements to the frequency-based winner (v001) by exploring tie-breaking strategies, counter saturation behavior, and computational costs at larger scales. The zone-based approach (v002) is a viable alternative if simplicity matters more than the marginal performance gain. Drop the ghost-list concept unless a fundamentally different insertion rule can be justified; the middle-position insertion was the wrong choice.
