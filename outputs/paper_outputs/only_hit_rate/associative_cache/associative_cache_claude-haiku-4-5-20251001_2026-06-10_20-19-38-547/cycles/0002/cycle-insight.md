# Cycle 0002 Insight

## Cohort outcome

This cycle refined the proven stash mechanism from Cycle 0001 by testing capacity scaling, eviction ordering, and selective insertion filtering. v002's capacity doubling (10% → 25%) emerged as the sole winner, validating that expanding the recovery window captures more collision-driven evictions—even though the gain was modest relative to capacity increase, suggesting diminishing returns are setting in.

## Patterns

- **Capacity scales faster than sophistication**: v002's straightforward capacity increase decisively outperformed both v001's LRU-ordering refinement and v003's occupancy-aware filtering. This indicates the bottleneck is raw buffer size, not optimization of which items to preserve within a bounded stash. The stash's insertion selectivity and eviction order are secondary to having enough space to capture working-set re-use.

- **Stash mechanism holds its own across designs**: All three variants treated the stash as the primary lever for improvement—no architectural changes, no geometry adjustments. This validates Cycle 0001's direction: post-eviction recovery via stash is the productive path forward.

- **Diminishing returns appear early**: v002's 2.5× capacity increase yielded only 1.2% hit-rate gain. This pattern suggests the highest-value evictions (those with immediate or near-term re-use) are captured by the baseline 10% stash, and larger buffers increasingly capture lower-probability re-accesses.

- **Filtering reduces churn without winning**: v003 demonstrated that occupancy-aware insertion filtering improves hit rate by avoiding low-value captures—yet it did not outperform v002. This suggests filtering effectiveness may depend on orthogonal factors like total capacity or eviction order.

## Open questions

- At what capacity does the stash saturate relative to the workload's actual re-use profile? Is the sweet spot between 15–25%, or does scaling beyond 25% still yield gains?
- Why didn't v003's filtering mechanism compound with v002's larger capacity in a direct head-to-head test? Would combining both improvements exceed either individually?
- Does stash ordering matter more as capacity grows? LRU showed no benefit at 10% capacity—would it unlock gains at 25% or higher?

## Continuation

v002's win confirms that stash capacity remains a productive axis. The modest 1.2% gain suggests the workload's re-use window is being progressively captured as capacity expands, but no single size is saturating. Next cycle should probe the capacity-filtering trade-off directly: implement a hybrid combining v002's larger stash (25%) with v003's occupancy-aware selective insertion to test whether filtering becomes more valuable at higher capacities. Also explore intermediate capacity points (15–20%) to locate the efficiency frontier. Deprioritize further eviction-ordering refinements unless the next cycle's capacity explorations flatten or unless filtering reveals a clear synergy.
