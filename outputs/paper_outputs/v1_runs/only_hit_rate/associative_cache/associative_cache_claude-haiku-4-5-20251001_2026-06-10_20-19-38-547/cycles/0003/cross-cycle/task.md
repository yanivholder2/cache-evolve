# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

This cycle explored three strategies to reduce collision-driven eviction losses: a bounded stash buffer for post-eviction recovery, inter-bucket item redistribution via neighboring-bucket probing, and geometry rebalancing toward wider buckets. Only the stash buffer (v001) succeeded, achieving a measurable hit-rate improvement by capturing evicted items and providing second-chance lookup on misses.

## Patterns

- **Recovery beats redistribution within architectural constraints**: v001's simple mechanism to re-capture evicted items outperformed v002's more sophisticated inter-bucket probing, which failed because the cache's slot_legality invariant prohibits bucket migration. This suggests the cache's design already enforces strict placement rules that leave little room for dynamic redistribution.

- **Evicted items exhibit temporal re-use**: v001's success validates that a significant fraction of discarded items are actually re-accessed, making them worth preserving in a bounded overflow buffer rather than discarding immediately. This workload characteristic is a key finding.

- **Geometry is a blunt tool**: v003's collision-reduction approach (fewer, wider buckets) was not a winner despite reducing collision probability, suggesting that longer per-bucket scans or other trade-offs outweigh the collision benefit, or the workload doesn't have sufficient collision pressure for geometry alone to matter.

## Open questions

- Why didn't v003's collision-reducing geometry win despite its targeted optimization? Is the cost per-bucket linear scan, or is collision pressure not the actual bottleneck?
- Could a hybrid approach combining v001's stash mechanism with v003's geometry improvements yield better results than either alone?

## Continuation

The stash mechanism proved that post-eviction recovery is the right direction. Next cycle should explore variations on the stash itself: different eviction policies (LRU vs. FIFO), stash sizing trade-offs, and whether more sophisticated second-chance structures (e.g., tiered stashes or victim caches) can extend the gains. Deprioritize geometry changes and inter-bucket schemes unless the next cycle reveals collision is still the dominant loss mechanism.


### Cycle 0002

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


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

This cycle attempted to advance the stash mechanism via three directions: filtering refinement at proven 25% capacity (v001), and capacity saturation exploration at 20% and 30% (v002, v003). Only v001 compiled and executed; it confirmed that occupancy-aware selective insertion adds no value on top of the existing 25% capacity baseline. The two capacity-exploration variants encountered the same Rust borrow checker constraint, preventing direct measurement of the efficiency frontier and saturation boundary.

## Patterns

- **Filtering hits a ceiling at 25% capacity**: v001's selective insertion—designed to avoid capturing low-value evictions from under-occupied buckets—yielded zero improvement. This validates the Cycle 0002 insight that raw capacity is the binding constraint; the 25% stash already captures all re-use-worthy items, leaving no room for filtering to add value. Items evicted from lightly-loaded buckets are naturally discarded by FIFO anyway, making proactive filtering redundant.

- **Architectural constraints block capacity exploration**: Both v002 (20%) and v003 (30%) failed at the build gate with identical borrow checker errors. This suggests the stash scaling axis—already proven productive through 25%—may hit hard architectural limits when attempting further refinements or explorations. The constraint lies in how stash access is sequenced within a single operation.

- **Stash-centric optimization shows diminishing returns**: After three cycles focusing exclusively on stash tuning (size, eviction order, insertion filtering), the last successful improvement (Cycle 0002's capacity doubling) yielded only 1.2% hit-rate gain. v001's null result indicates saturation is near or already reached.

## Open questions

- What is the true capacity saturation point (15%, 20%, 25%, or beyond)? The build failures prevented testing v002 and v003, leaving the efficiency frontier unmapped.
- Do the borrow checker errors reflect fundamental architectural constraints in the stash design, or can they be resolved with refactoring to enable further exploration?
- Has the stash mechanism exhausted its potential, or are entirely different levers (cache geometry, eviction policy outside the stash, workload adaptation) the next productivity axis?

## Continuation

v001's lack of improvement is instructive: filtering is unhelpful once capacity reaches 25%, confirming the working set is already fully captured. The build failures for v002 and v003 reveal an architectural bottleneck that blocks direct capacity saturation testing. Next cycle should either (1) resolve the borrow checker constraint through refactoring to enable systematic capacity exploration between 15–30%, or (2) shift focus away from stash tuning entirely—the mechanism has matured and stash-only refinements are showing zero or minimal gains. If pursuing (2), consider probing orthogonal axes: whether collision clustering is addressable at the geometry level, whether the workload exhibits phase-dependent behavior worth adapting to, or whether the cache's fundamental layout constrains further progress.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
