# Cross-Cycle Insights

## Recurring wins

- **Stash mechanism as post-eviction recovery remains the foundational win.** Every winning variant from C0001 through C0004 preserved or extended the bounded stash buffer. It consistently outperformed architectural alternatives (inter-bucket probing, pure geometry) because it directly captures evicted items that exhibit temporal re-use—a workload characteristic validated across all cycles.

- **Capacity is the dominant lever within stash optimization.** C0002's capacity doubling decisively outperformed both eviction-ordering refinement (v001) and occupancy-aware filtering (v003). This established that raw buffer size is the binding constraint; once capacity reaches 25%, further policy sophistication (LRU ordering, selective insertion) yields zero or minimal gains.

- **Orthogonal levers restore productivity after single-axis saturation.** After stash-only refinements plateaued (C0002 → C0003: 1.2% → 0% improvement), pivoting to geometry in C0004 (v001: geometry + 25% stash) unlocked fresh gains (0.26%). Neither axis alone suffices; they address complementary loss mechanisms—geometry reduces collisions feeding the stash; stash captures what does evict.

## Recurring failures

- **Inter-bucket redistribution is architecturally blocked.** C0001's v002 failed because the cache's slot_legality invariant prohibits bucket migration. This rejection was definitive; no subsequent cycle revisited item mobility across buckets.

- **Eviction-order refinement beyond FIFO shows minimal ROI.** LRU ordering on stash eviction was tested twice (C0002 v001, C0004 v002) and trailed both capacity scaling and geometry improvements both times. After capacity reaches 25%, eviction order is a second-order lever—the working set is already fully captured, leaving little discrimination opportunity.

- **Filtering and occupancy-aware insertion filtering are redundant at mature capacity.** C0003 v001 (occupancy-aware filtering at 25%) yielded zero improvement; items from under-occupied buckets are already naturally discarded by FIFO. This pattern confirms that capacity-based recovery dominates decision-making once sufficient space exists.

- **Stash-only tuning saturates rapidly.** The progression C0001 → C0002 (1.2% gain) → C0003 (0% gain) shows diminishing returns. Capacity scaling from 10% to 25% was the only stash-internal win; subsequent refinements (filtering, ordering, further capacity exploration) added nothing, signaling architectural limits or working-set saturation.

## Open questions

- **What is the true capacity saturation point?** C0003's build failures prevented testing 20% and 30%, leaving the efficiency frontier unmapped. Is the sweet spot at 25%, or could 15–20% capture 90% of the gain at lower cost?

- **Can geometric rebalancing be extended further?** C0004's 0.26% improvement from wider-bucket rebalancing is modest. Different bucket-width ratios or collision-reduction strategies might unlock higher gains, or the current design may already approach the geometry ceiling for this workload.

- **Have we hit the natural exploitable ceiling for this workload-cache pairing?** Gains are shrinking (1.2% → 0% → 0.26%) across four cycles. Does the combination of 25% stash + current geometry saturate the workload's re-use opportunities, or do orthogonal axes (workload phase detection, alternative replacement policies, adaptive cache tuning) remain untapped?

- **What architectural constraints block capacity exploration beyond 25%?** C0003's borrow checker errors on v002 and v003 prevented systematic saturation testing. Are these hard limits in the current design, or can refactoring enable broader exploration?

## Per-policy notes

- **FIFO eviction is the implicit baseline and remains competitive.** Most variants operate within FIFO + capacity framework. No policy-focused exploration has systematically compared FIFO to alternatives other than incidental LRU testing.

- **LRU ordering on stash shows secondary promise but cannot match geometric or capacity improvements.** Tested in C0002 v001 and C0004 v002, LRU consistently underperformed both capacity scaling and geometric rebalancing. This suggests eviction order matters only when working-set is not fully captured; once capacity suffices, order is immaterial.

- **Geometry and capacity are orthogonal and mutually reinforcing.** C0004's winner (geometry + 25% stash) outperformed C0004 v003 (geometry + 15% stash), confirming both dimensions are binding constraints. Geometry reduces collision-driven eviction pressure; capacity absorbs the remaining evictions. Neither alone compensates for the other.
