# Cross-Cycle Insights

## Recurring wins

- **Capacity and resource allocation beat filtering and selectivity**: Every variant that relied on filtering to decide what to keep (occupancy-aware, collision-aware, access-count thresholds across Cycles 0003–0007) failed or regressed. Every successful variant increased raw capacity or rebalanced allocation (stash doubling in Cycle 0002, two-tier promotion in Cycle 0006, asymmetric partition in Cycle 0007). The workload has no obvious "low-value" evictions; nearly all re-accessed items are worth recovering.

- **Orthogonal levers unlock productivity after saturation within an axis**: Cycles 0002–0003 showed stash tuning alone plateauing; Cycle 0004's shift to geometry + stash pairing broke through. Cycles 0004–0005 showed geometry refinement hitting a sharp peak; Cycle 0006's pivot to stash organization (two-tier) resumed gains. The pattern is consistent: when one axis converges, switching to a new constraint is more productive than refinement within the saturated axis.

- **Simple unconditional mechanisms outperform complex selective ones**: Unconditional FIFO insertion at 25% capacity (Cycle 0002) beat occupancy-aware filtering; promotion-on-hit without gating (Cycle 0006) beat promotion filtering (Cycle 0007 v002); static asymmetric partitioning (Cycle 0007) beat counter-based dynamic adjustment. The workload rewards straightforward, full-capacity utilization over discriminating heuristics.

- **Two-tier stash organization is a sustained productivity axis**: Introduced in Cycle 0006 and refined in Cycle 0007, the promotion-on-hit mechanism with partition tuning has shown measurable gains while remaining architecturally simple. The two-tier design is robust to refinement and enables incremental progress through resource reallocation.

## Recurring failures

- **Filtering at insertion or promotion consistently fails**: Cycles 0003, 0005, 0006, 0007 all tested filtering variants—occupancy-aware, collision-aware, access-count thresholds, and gated promotion—with no wins. The underlying insight is stable across cycles: the workload does not exhibit clear separation between high- and low-value evictions. Filtering discards potentially recoverable items without reducing stash pressure; allocation is the binding constraint.

- **Counter and item-level manipulation for phase response does not translate**: Cycles 0006–0007 detected valid phase transitions in the workload (collision-distribution divergence) but attempted to couple phase detection with counter resets or counter amplification. Both approaches failed. Detection works; response at the item-counter level does not. The disconnect suggests phase transitions may require coarser-grained policy changes (e.g., partition rebalancing or geometry adaptation) rather than per-item state manipulation.

- **Borrow checker blocks capacity saturation exploration**: Cycles 0003 and 0005 failed to build variants testing 20% and 30% capacity, preventing direct measurement of saturation boundaries. This architectural constraint has persisted across three attempts and remains unresolved, leaving the true capacity sweet spot unmapped.

- **Geometry-only and fine-grained geometric refinement saturate quickly**: Cycle 0001's pure geometry didn't win; Cycle 0005's 0.75 factor overshot; Cycle 0006's fine-grained search showed 0.71 is a sharp peak with no tuning zone nearby. Geometry is highest-impact when paired with stash (Cycle 0004, +0.26%); standalone refinement has limited room.

## Open questions

- **What is the partition tuning frontier?** Cycle 0007's 18/7 asymmetric split (hot/cold) succeeded, but is it near-optimal or is there further headroom at nearby ratios (e.g., 15–20% hot, 5–10% cold)?

- **Are there organizational strategies beyond two-tier promotion that better exploit workload structure?** Frequency-based tiering, multi-level hierarchies, or predictive prefetching to hot tier remain untested. The current mechanism promotes on re-access; could anticipatory promotion or time-windowed recency yield additional gains?

- **Can phase detection couple with effective response mechanisms?** Phases exist and are detectable; counter manipulation does not help. Would dynamic partition rebalancing (expanding hot tier during high-collision phases) or geometry adaptation during transitions produce gains without architectural friction?

- **Has the two-tier design reached its local optimum, or are there organizational alternatives?** After partition tuning, refinement may yield only marginal gains. Testing frequency-based promotion rules or multi-level tiering could reveal whether the current design's structure is optimal or whether a different organization better matches workload access patterns.

- **Are we approaching a hard ceiling on exploitable gains?** Seven cycles of incremental progress (0.26% in Cycle 0004, smaller gains since) and convergence across multiple axes suggest the workload-cache pairing may be reaching saturation. Exploring fundamentally different levers (alternative bucket layouts, global cache replacement policies beyond FIFO, workload-phase-aware adaptation with coarse-grained policy change) may be required for larger breakthroughs.

## Per-policy notes

- **FIFO remains the consistent eviction policy**: The stash has used FIFO (or unconditional insertion without explicit ordering) across all seven cycles. LRU was tested once in Cycle 0002 on the stash itself and lost to baseline FIFO, suggesting temporal order preservation within the stash is already well-matched to the workload's re-access clustering. No global cache replacement policies (e.g., LRU, LFU across the main hash table) have been tested or modified; optimization has been confined to the stash and geometry.

- **Two-tier promotion-on-hit is a sophisticated FIFO variant, not a departure**: Cycles 0006–0007's multi-tier stash with promotion on re-access can be viewed as hierarchical FIFO with recency-based promotion between tiers. It retains FIFO's simplicity and order-preservation while adding a learning layer (distinguishing cold from hot items via hit patterns). This suggests the cache's workload is well-suited to recency-based policies.

- **Asymmetric partition allocation reveals skewed re-access distribution**: Cycle 0007's success with 18% hot / 7% cold implies the workload's evicted items have predictable, uneven re-access: most land in cold initially, but a concentrated subset (the 18% allocated to hot) re-access repeatedly. Static allocation reflecting this distribution outperforms dynamic counter-based adaptation, suggesting workload structure is stable enough to justify fixed resource allocation rather than adaptive mechanisms.
