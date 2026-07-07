# Cross-Cycle Insights

## Recurring wins

- **Stash buffer for post-eviction recovery**: All three cycles confirm the bounded stash mechanism is productive. Simple capacity expansion (Cycle 0002, 10%→25%) decisively outperformed both eviction-ordering refinements and occupancy-aware filtering, establishing that raw buffer size, not sophistication of what to preserve within the stash, is the binding constraint.

- **Simplicity beats complexity at the margin**: Straightforward capacity scaling has consistently outperformed tailored insertion filters, LRU orderings, or other refinements designed to be "smarter" about stash occupancy. The workload exhibits enough temporal re-use that the stash naturally captures the right items with a FIFO discipline.

- **Evicted items are genuinely re-accessed**: Across three cycles, the stash mechanism has validated that a measurable fraction of discarded items return within the recovery window. This persistent re-use pattern justifies the stash as a core architecture, not a speculative feature.

## Recurring failures

- **Insertion filtering and occupancy-aware selection provide no incremental gain**: v003 in Cycle 0002 (occupancy-aware filtering), v001 in Cycle 0003 (selective insertion at 25% capacity), and LRU-ordering (v001 in Cycle 0002) all failed to improve on baselines despite targeting identifiable inefficiencies. Once stash capacity reaches 25%, competing items for that space are already low-value by any ranking; filtering doesn't remove them because they self-discard via FIFO.

- **Geometry-based collision reduction alone does not win**: v003 in Cycle 0001 (rebalancing toward wider buckets) failed despite reducing collision probability, suggesting the per-bucket scan cost or other trade-offs dominate the collision benefit, or the workload lacks sufficient collision pressure to make geometry changes worthwhile at current stash sizes.

- **Architectural constraints block systematic exploration**: Cycle 0003's v002 and v003 variants failed at build time (borrow checker errors) when attempting to probe capacity saturation at 20% and 30%. This suggests the stash scaling axis may hit hard limits within the current design, preventing direct measurement of where capacity benefits plateau.

## Open questions

- **Where is the capacity saturation frontier?** Cycles 0002 and 0003 showed gains at 25% and a null result at 25%, but the borrow checker errors prevented testing 20% and 30%. The true saturation point—and the efficiency frontier between raw capacity and its cost—remains unmapped.

- **Are the borrow checker constraints fundamental architectural limitations, or can refactoring unlock further exploration?** Both failed variants in Cycle 0003 hit the same error, hinting at a sequencing bottleneck in stash access. Resolving this could re-open the capacity axis or enable entirely new lever experiments.

- **Has the stash mechanism exhausted its potential?** Cycle 0003's zero-gain result and Cycle 0002's diminishing returns (2.5× capacity for 1.2% improvement) suggest saturation may be near. If so, the next productivity axis may lie outside stash tuning: collision clustering at the geometry level, workload phase-dependency, cache layout constraints, or eviction policies independent of the stash.

- **Could hybrid approaches (stash + geometry) unlock further gains?** Cycle 0001 suggested the two might synergize, but no direct combined experiment was attempted. Wider buckets + larger stash might yield more than either alone.

## Per-policy notes

- **FIFO eviction within the stash performs as well as LRU**: v001 in Cycle 0002 (LRU ordering) added no benefit over the baseline FIFO stash, and v002 (capacity expansion, implicitly FIFO) won. This suggests the temporal locality in the workload's re-use pattern makes FIFO ordering sufficient; more sophisticated recency tracking does not pay off.

- **Stash-centric policies show convergence**: Across all three cycles, every attempted refinement—LRU, filtering, selective insertion—operated within the stash framework. None revisited the cache's core eviction policy (CuckooHash's collision resolution) or external policies beyond the recovery mechanism, implying the stash has become the primary lever in the experimental design.
