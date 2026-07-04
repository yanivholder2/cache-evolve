# Cross-Cycle Insights

## Recurring wins

- **Stash buffers for post-eviction recovery**: Simple bounded buffers that re-capture recently evicted items outperform more complex redistribution schemes. This approach succeeds because it works within the cache's existing architectural constraints rather than fighting them.

## Recurring failures

- **Inter-bucket item migration**: Attempts to dynamically redistribute items across buckets fail due to the cache's slot_legality invariant, which enforces strict per-item placement rules. This constraint is fundamental enough that sophisticated probing schemes cannot overcome it.
- **Collision reduction through geometry alone**: Rebalancing bucket dimensions to reduce collision rates did not translate to measurable wins, suggesting either per-bucket scan costs are linear, or collision pressure is not the actual bottleneck.

## Open questions

- Is the collision-geometry trade-off purely linear scan cost, or is collision not the dominant loss mechanism in the workload?
- Can hybrid approaches combining stash recovery with geometry improvements unlock gains neither achieves alone?
- What eviction policy (LRU vs. FIFO) maximizes effectiveness within the stash mechanism?

## Per-policy notes

- Eviction policy choice remains unexplored; candidate policies (LRU, FIFO) should be evaluated within the stash framework before committing to geometry changes.
- Workload exhibits significant temporal re-use of discarded items, suggesting that preservation mechanisms (stashes, victim caches) are worth prioritizing over collision-avoidance schemes.
