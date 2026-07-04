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


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

This cycle marked a strategic pivot: after three cycles of stash-only refinement, geometric rebalancing re-entered the optimization space. The winner combines wider-bucket geometry with the proven 25% stash to attack collisions at their source while retaining strong recovery coverage. v001 achieved a 0.26% hit-rate improvement, validating that geometry and stash are complementary levers—neither alone suffices, but both together compound.

## Patterns

- **Orthogonal levers restore productivity after saturation**: Cycles 0002–0003 showed that stash tuning alone (capacity scaling, filtering, eviction ordering) faced diminishing returns. v001's success demonstrates that pivoting to geometry—an axis deprioritized after Cycle 0001—unlocks fresh gains. Wider buckets reduce collision-driven evictions feeding the stash, reducing demand on the recovery mechanism itself.

- **LRU ordering trails geometry in impact**: v002's LRU stash eviction outperforms baseline but underperforms v001's geometric rebalancing. This confirms that after three cycles of stash-internal optimization (capacity, filtering, now ordering), further refinements within the stash have limited upside. Collision reduction is now the more productive axis.

- **Both dimensions are binding constraints**: v003's attempt to trade stash capacity (15%) for geometry gains fails to match v001 (25% + geometry). The cache requires both levers operating together; geometry cannot fully compensate for undersized recovery coverage. This rejects the Cycle 0003 hypothesis that collision reduction could reduce stash overhead requirements.

- **Gains remain modest but consistent**: All three variants improved over baseline, with v001 winning by a narrow margin. The pattern mirrors Cycles 0002–0003, where each optimization achieves small incremental progress. The workload appears to have limited exploitable headroom remaining.

## Open questions

- What is the saturation point for geometric rebalancing? Are the 0.26% gains from the current bucket-width adjustment the peak, or could refined rebalancing ratios (e.g., different width-to-count trade-offs) push further?
- Does the synergy between geometry and 25% stash plateau at the current design, or is there room to optimize their interaction (e.g., adaptive geometry based on collision rate)?
- Have we reached the natural ceiling for this workload-cache pairing, or do entirely different optimization axes (workload adaptation, alternative eviction policies) remain unexplored?

## Continuation

v001 validated that geometry and stash are mutually reinforcing—the 0.26% gain, though modest, breaks the stash-only plateau and signals a productive direction. v002's LRU ordering shows secondary promise but cannot match geometric improvements; v003's stash-capacity trade-off definitively failed. Next cycle should either (1) refine geometric rebalancing further (test different bucket-width ratios and collision-reduction strategies to locate a higher-productivity region), or (2) assess whether the combined geometry + 25%-stash design has saturated the workload's exploitable margin. If geometric refinement yields only marginal gains, consider stepping back to evaluate whether fundamentally different levers (workload phase detection, alternative cache replacement policies beyond LRU, or redesigned bucket layouts) are required to unlock larger improvements.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

This cycle tested refinements at the geometric and filtering boundaries of the proven design (0.71 reduction factor + 25% stash). Both directions regressed: increasing geometric aggressiveness to 0.75 overshot the collision-reduction optimum, and collision-aware stash filtering eliminated valuable recoveries without net gain. v003's relative victory (despite hurting hit-rate) reflects convergence to a local optimum; the parent design appears to be the better-positioned point in the combined geometric-capacity-filtering design space.

## Patterns

- **Local optimum is stable**: The parent design's 0.71 geometric factor and unconditional 25%-capacity stash insertion represent a local equilibrium. Pushing geometry more aggressive (0.75) and filtering more selective (collision-aware threshold) both moved away from peak performance, indicating the system has settled into a well-balanced configuration with limited room for refinement within these axes.

- **Unconditional capture remains optimal**: v003 tested collision-aware selective insertion, filtering evictions from low-contention buckets to reduce stash pollution. The result contradicts the hypothesis—blanket FIFO insertion at 25% capacity captures all meaningful re-uses, and collision frequency does not predict recovery value. This replicates Cycle 0003's occupancy-filtering finding with a different signal; filtering is not a productive axis regardless of the signal chosen.

- **Geometric refinement has tightened bounds**: v001's regression when pushing from 0.71 to 0.75 confirms that collision-reduction gains have saturated. The design space is now constrained; v002 from Cycle 0004 (0.71) sits near the peak, and both higher and lower directions yield diminishing or negative returns. Further geometric exploration requires narrower, more expensive searches.

- **Architectural constraints continue to block capacity exploration**: v002's build failure (borrow checker error on stash insertion sequencing) persists as the third instance across cycles. Stash capacity scaling beyond 25%—a direction validated earlier—remains technically blocked, leaving the capacity saturation frontier unmapped.

## Open questions

- Is the geometric optimum precisely at 0.71, or is there a fractional refinement (0.68–0.70 or 0.72–0.73) that yields incremental gains within a narrower range?
- Can the architectural constraint preventing stash capacity scaling be resolved through refactoring, or does it reflect a fundamental limit of the bucket-level locking model?
- Has the workload-cache pairing exhausted its exploitable margin, or do entirely different levers (workload phase adaptation, eviction policies orthogonal to FIFO, collision-reduction through alternative bucket-layout strategies) remain?

## Continuation

v003's win, though accompanied by regression, confirms convergence to a local optimum around 0.71 and 25% capacity. Both geometry refinement (pushing to 0.75) and filtering (collision-aware threshold) moved away from peak, indicating the parent design is well-positioned. Next cycle should pursue one of two paths: (1) **Fine-grained local search**: test geometric factors in the 0.68–0.73 range at smaller increments to locate the precise peak and verify whether small adjustments yield marginal gains within the convergence zone, or (2) **Escape the local optimum**: accept that stash-geometry tuning has saturated and pivot to fundamentally different optimization axes (workload phase detection and dynamic adaptation, alternative collision-reduction strategies orthogonal to bucket width, or adaptive eviction policies beyond FIFO). Resolve the borrow checker constraint if pursuing stash-capacity exploration, but deprioritize it unless fine-grained local search confirms the need for larger capacity to support new geometric configurations.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

This cycle tested three escape routes from the Cycle 0005 local optimum (0.71 reduction factor + 25% stash): fine-grained geometric search, stash internal organization, and phase-aware adaptive tuning. Only v002—a two-tier stash with promotion-on-hit—produced hit-rate gains. The winner reveals that stash *organization* (how to treat items already captured) is more productive than *filtering* (what to capture) or *detection without response* (phase sensing without effective policy change).

## Patterns

- **Fine-grained geometric search has converged**: v001's regression when tuning from 0.71 to 0.70 confirms that the collision-reduction optimum is a sharp peak, not a tuning zone. The Cycle 0005 hypothesis for narrower local search was incorrect; 0.71 appears to be a hard boundary rather than a starting point for refinement. Geometry-only improvements have exhausted themselves.

- **Stash organization outperforms stash filtering**: Cycles 0003 and 0005 tested filtering strategies (occupancy-aware, collision-aware insertion) with no gain; v002 takes a different path by partitioning the 25% capacity into hot and cold tiers and promoting items on re-access. This learned differentiation improves hit rate without adding architectural constraints or filtering logic. The stash has capacity to spare—the issue is how to use it, not what to reject.

- **Phase detection requires an effective response mechanism**: v003 correctly identified workload phase transitions via collision-distribution divergence, but pairing phase detection with counter resets failed to improve hit rates. Detection-without-response provides diagnostic clarity but no performance gain. The phase-adaptation axis remains open but requires a different coupling strategy (e.g., dynamic tier sizing or geometry rebalancing during transitions, rather than counter manipulation).

- **Stash-geometry pairing is settling into maturity**: The 0.71 factor + 25% capacity + hierarchical organization (v002) represents a three-layer design where collisions are managed geometrically, recovery is provided uniformly, and cache utility is optimized via stash internal structure. This convergence mirrors Cycles 0004–0005 patterns; further gains now depend on secondary refinements within existing levers rather than exploring new axes.

## Open questions

- How sensitive is the two-tier stash to partition ratios? Would different splits (e.g., 10% hot / 15% cold or 15% / 10%) yield further gains, or is 12.5/12.5 near-optimal?
- Are there alternative stash-organization strategies (e.g., frequency-based aging, time-windowed recency, predictive re-use indicators) that outperform promotion-on-hit?
- Can phase detection be coupled with effective response mechanisms beyond counter resets—e.g., dynamic tier sizing, geometric adaptation, or stash capacity rebalancing during transitions?

## Continuation

v002 broke through the Cycle 0005 plateau by shifting focus from *filtering* (what to keep) to *organizing* (how to prioritize within kept items). The two-tier promotion-on-hit mechanism adds no architectural overhead and shows measurable gains. v001 and v003 confirm that fine-grained geometric search and phase-detection-only strategies are not productive paths. Next cycle should refine the two-tier stash via parameter exploration (tier ratios, promotion thresholds, aging within tiers) to locate the next local optimum, or explore orthogonal organizational strategies (frequency-based prioritization, multi-level tiering, predictive prefetching to the hot tier). If refinement yields only marginal gains, revisit phase detection with alternative response mechanisms (tier-sizing adaptation, geometry rebalancing) that couple effective diagnosis with productive policy change.


### Cycle 0007

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

This cycle tested three directions from the Cycle 0007 asymmetric-partition baseline: further partition tuning (20% hot / 5% cold), staged multi-tier promotion with frequency gating, and simplification via unified flat stash. Only v001 succeeded with marginal hit-rate gains, confirming the two-tier asymmetric design is resilient but approaching saturation in partition space.

## Patterns

- **Partition tuning is nearing saturation**: v001's minimal improvement from the 18/7 baseline to 20/5 suggests the workload's re-access distribution is already well-matched. Further hot-tier expansion yields diminishing returns, indicating the design may be approaching an allocation boundary within this two-tier strategy.

- **Two-tier promotion-on-hit is a load-bearing mechanism**: v003's regression (consolidating to a single unified 25% stash) and v002's regression (staged cold→warm→hot promotion) both confirm that the two-tier structure is not organizational overhead. The mechanism provides the feedback signal identifying which evicted items deserve premium hot placement; removing it sacrifices this critical learning capability.

- **Gatekeeping and staged promotion filter out valuable items**: v002's failure when requiring two re-accesses for hot-tier promotion replicates the cross-cycle pattern: filtering logic—even frequency-based—delays or prevents promotion of genuinely useful single-re-access items. The workload rewards rapid escalation over observation; items demonstrating one re-access should immediately reach hot tier.

- **The design convergence is tightening**: Cycles 0006–0008 show that stash *organization* (two-tier promotion-on-hit) outperforms filtering, gatekeeping, or simplification. The 20/5 asymmetric partition with immediate promotion represents a mature equilibrium where geometric collision reduction (0.71 factor), uniform capacity provision (25% total), and adaptive organization (two-tier with asymmetric allocation) are locked in place.

## Open questions

- Has partition tuning been exhausted at 20/5, or do fractional refinements (19%, 21%, etc.) yield further gains within a narrower range?
- Can phase detection finally be coupled with effective *dynamic response*—e.g., rebalancing partition ratios during workload transitions—rather than counter manipulation?
- Are there organizational strategies orthogonal to asymmetric partitioning that exploit workload structure—e.g., frequency-based aging, time-windowed recency, multi-level tiering without staged gating?

## Continuation

v001's marginal gain confirms partition tuning is subject to diminishing returns; v002 and v003 demonstrate that multi-tier staged promotion and unified simplification are counterproductive. The two-tier asymmetric-partition design (now at 20/5) is proven and stable. Next cycle should pursue one of two paths: (1) **Fine-grained partition refinement**: test fractional adjustments (19%, 21%, etc. hot-tier allocation) to verify whether the local optimum is sharp or whether marginal gains remain, or (2) **Escape via alternative organization**: shift focus from partition tuning to orthogonal mechanisms (frequency-based aging, multi-level hierarchies without gatekeeping, or phase-aware dynamic rebalancing) that adapt the stash beyond static allocation. If partition refinement yields only noise-level gains, accept convergence at 20/5 and prioritize phase-aware dynamic response coupling as the next escape route.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

This cycle tested three refinements to the two-tier asymmetric-partition design (20% hot / 5% cold baseline): fractional partition tuning (19% / 6%), age-tracked LRU eviction within the hot tier, and phase-aware dynamic partition rebalancing. Only v002—LRU eviction in the hot tier—delivered measurable gains (+0.5% hit-rate). The winner reveals that within-tier eviction order matters for temporal locality, but gains are modest; the cycle also confirms that partition space is exhausted and phase-aware rebalancing faces implementation barriers.

## Patterns

- **Partition refinement has saturated**: v001's null result (19/6 matching 20/5 performance) directly contradicts the hypothesis that the 20/5 optimum admits marginal gains via narrower local search. The workload's re-access distribution is already well-matched to this allocation; 1-percentage-point shifts yield no improvement. This closes out the partition-tuning axis opened in Cycle 0007 and confirms 20/5 as a stable local minimum, not a sharp boundary artifact requiring correction.

- **Eviction policy within tiers is a secondary lever with diminishing returns**: v002's +0.5% gain (LRU vs. FIFO in hot tier) proves that temporal recency of individual item access patterns influences performance, but the gain is small. This indicates the two-tier organization itself (promoting re-accessed items to hot) already captures the bulk of temporal locality; refining *which* hot-tier item to evict addresses a narrower signal. The core mechanism is robust, but optimization within it yields marginal returns.

- **The design has reached architectural maturity**: Cycles 0006–0009 form a convergence arc: core mechanism proven (two-tier promotion-on-hit), partition allocation optimized (18/7 → 20/5 → confirmed), and now internal eviction ordering refined (+0.5% gain). Each new axis yields smaller returns, and the design space appears to be tightening around a local equilibrium. No architectural breakthrough is visible within the current two-tier framework.

- **Phase-aware dynamic response remains unproven and implementation-constrained**: v003's crash (panic in Vec::remove during simulation) prevents testing the hypothesis that phase detection coupled with structural partition rebalancing succeeds where counter manipulation failed. The failure suggests either the implementation strategy was flawed or the integration with the two-tier mechanism introduces unexpected constraints. This axis remains open but requires either rethinking the coupling strategy or deferring further phase-aware exploration.

## Open questions

- Does the +0.5% LRU gain hold when combined with other stash refinements, or is it orthogonal to partition tuning? Could LRU + asymmetric partitioning establish a new local optimum beyond 20/5 + FIFO?
- Are there organizational strategies orthogonal to partition allocation and within-tier eviction that could unlock larger gains—e.g., multi-level tiering without gatekeeping, frequency-based aging, or predictive stash prefetching?
- Can phase-aware dynamic rebalancing be revisited with a simpler integration strategy (e.g., no re-partitioning of existing items, only forward-looking allocation adjustments) to avoid implementation crashes while still coupling detection with structural response?

## Continuation

v002's modest +0.5% gain confirms that optimization within the two-tier framework continues to yield returns, but only incrementally. v001's null result definitively closes partition tuning; 20/5 is the stable optimum for this workload. v003's crash indicates phase-aware rebalancing is architecturally complex or poorly integrated. Next cycle should pursue one of two paths: (1) **Accept convergence at the current local optimum** (20/5 + LRU, or 20/5 + FIFO if LRU proves unstable), document the design as mature, and shift focus to orthogonal improvements outside the two-tier framework (frequency-based tiering, predictive mechanisms, or entirely different eviction strategies), or (2) **Escape via orthogonal organization**: test frequency-based multi-level tiering, time-windowed recency variants, or stash-prefetching strategies that operate within the 25% recovery capacity without modifying partition structure. If phase-aware dynamic rebalancing is pursued, simplify the implementation (e.g., defer item re-partitioning, adapt allocation only on phase transitions without moving existing stash contents) and integrate more conservatively with the promotion-on-hit mechanism.


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
