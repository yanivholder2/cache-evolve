# Cycle 0010 Insight

## Cohort outcome

This cycle tested whether the baseline three-tier architecture's 0.684 plateau could be broken by surgical refinements: eviction selection criterion (LRU vs FIFO), stash capacity expansion, and per-bucket saturation detection. Only the stash expansion variant (v002) improved hit-rate, validating that the baseline's 4-bucket stash was a genuine bottleneck in the promotion decision pipeline, not a superfluous tier. The other two refinements—eviction ordering and saturation-triggered cascades—proved inert or counterproductive, reinforcing that the ceiling is not defended by eviction selection or occupancy-driven cascading, but by capacity allocated to characterization.

## Patterns

- **Stash tier capacity, not tier count, may be the primary lever**: v002's expansion from 4 to 8 buckets (12.5% → 25% of total capacity) improved hit rate on the baseline three-tier architecture, directly reversing cycles 0007–0009's conclusion that stash allocation was balanced. This suggests the baseline was under-provisioning the filtering window, forcing items into promotion decisions without sufficient evidence of hotness. The gain contradicts the assumption that cycles 0008–0009's breakthrough came purely from collapsing the three-tier to two-tier; stash capacity may have been an equal or primary contributor.

- **Eviction selection criterion remains orthogonal to hit-rate**: v001's null result from switching LRU to FIFO on the two-tier (cycle 0008/v001) extends the finding across nine prior cycles: whether items are selected for eviction by recency or insertion order does not constrain hit-rate. The bottleneck is upstream in tier structure and promotion candidacy, not downstream in eviction ordering.

- **Per-bucket saturation as an eviction trigger is a false signal**: v003's degradation from proactive eviction at 80% bucket occupancy confirms that bucket-level fill signals do not correlate with item utility. Aggressive saturation-triggered eviction shrinks the effective working set before items' utility windows naturally close, yielding worse hit-rate than letting occupancy stabilize and eviction occur naturally within the tier's cascade structure.

- **The baseline three-tier architecture harbored an untested capacity rebalancing opportunity**: Prior cycles' exploration of the three-tier focused on insertion position, occupancy routing, eviction priority, and cascade structure, but accepted the 4-bucket stash allocation as given. v002's win reveals that capacity allocation itself was under-optimized in the baseline, suggesting cycles 0002–0007's parametric plateau was partially caused by an inadvertently constrained filtering tier, not purely by tier topology.

## Open questions

- Does v002's stash expansion on the three-tier achieve hit-rate parity with or exceed the two-tier collapse from cycles 0008–0009? If so, the simpler three-tier with better capacity allocation outranks the two-tier simplification.

- If v002 and the two-tier are equivalent, is the three-tier with expanded stash more maintainable or does the two-tier's simpler topology justify the same performance cost?

- Why was the 4-bucket stash acceptable through cycles 0002–0007's parametric tuning, yet becomes a bottleneck when isolated in the baseline? Is this a measurement artifact, or does context (prior architectural choices, parent state) mask capacity constraints?

## Continuation

v002's stash expansion breakthrough reopens the three-tier architecture as a viable path forward. For the next cycle, directly compare v002 (three-tier with 8-bucket stash) against the two-tier winner from cycle 0009 to determine whether simpler topology or better capacity allocation is the primary driver of cycles 0008–0009's gains. If v002 ties or beats the two-tier, make the three-tier with expanded stash the new baseline and abandon the two-tier. If the two-tier remains superior, test whether expanding its stash (if applicable) compounds gains further. Regardless, confirm that eviction ordering and occupancy-triggered cascades are exhausted as levers—v001 and v003's failures reinforce that refinements must target tier capacity and promotion routing, not eviction mechanics or saturation monitoring.
