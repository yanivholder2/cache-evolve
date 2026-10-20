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
