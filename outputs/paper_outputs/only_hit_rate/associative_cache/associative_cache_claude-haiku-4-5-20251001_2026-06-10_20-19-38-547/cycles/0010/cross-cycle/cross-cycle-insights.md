# Cross-Cycle Insights

## Recurring wins

- **Stash recovery as foundation**: Across all 10 cycles, the core mechanism of capturing evicted items in a bounded overflow buffer has proven consistently productive. Every winning variant (v001 in cycles 0001, 0002, 0004, 0006, 0007, 0008, 0010) either uses or improves upon this principle, validating that post-eviction recovery addresses a real workload characteristic: temporal re-use of discarded items.

- **Orthogonal axes break saturation**: When one optimization axis (e.g., stash capacity in cycle 0002, partition tuning in 0007–0009) approaches diminishing returns, pivoting to an orthogonal dimension (geometry in 0004, stash organization in 0006, promotion prioritization in 0010) unlocks fresh gains. The cache and workload present multiple, independently exploitable levers rather than a single constraint.

- **Two-tier promotion-on-hit mechanism is load-bearing**: Introduced in cycle 0006 and refined through 0009, this mechanism (cold tier as buffer, hot tier as premium recovery) consistently outperforms simpler designs. Removing it (cycle 0008 v003) or adding gatekeeping that delays promotion (cycle 0008 v002) both regress. The mechanism's core—promoting re-accessed items to priority status—is essential to extracting workload value.

- **Intelligent per-item prioritization beats static allocation**: Cycle 0010's frequency-weighted promotion scoring achieved +5.0% hit-rate gain by ranking evicted items by re-access likelihood rather than allocating capacity uniformly. This represents a paradigm shift from "optimize how much capacity each tier gets" (cycles 0007–0009, marginal gains) to "optimize which items get premium recovery" (cycle 0010, breakthrough gain).

## Recurring failures

- **Filtering and gatekeeping don't address the real problem**: Cycles 0003, 0005, 0006, 0007, 0008 tested filtering logic at multiple layers—occupancy-aware, collision-aware, access-count-based, and staged promotion thresholds. None delivered gains. The workload has no "pollution problem" requiring rejection of low-value items; instead, it shows skewed re-access distribution that benefits from prioritization, not filtering. Gatekeeping delays promotion of valuable items and trades hit-rate for reduced churn (cycle 0008 v002, net negative).

- **Counter manipulation for phase response is ineffective**: Cycles 0006 and 0007 correctly identified workload phase transitions via divergence in collision distributions, then attempted to respond via counter resets or boosts (amplifying hot-item counters). Both approaches yielded no performance gain. Detection divorced from structural policy change (e.g., partition rebalancing or geometry shifts) does not translate to improved eviction decisions.

- **Architectural complexity without targeted impact fails**: When additional tiers or mechanisms are proposed without addressing the core bottleneck, they regress or provide no gain. Cycle 0008's unification to a single 25% stash regressed (removing the two-tier organization's learning signal); cycle 0010's three-tier build failure suggests the design may over-constrain shared-tier transitions. Added complexity requires corresponding algorithmic value.

- **Geometry alone is insufficient; capacity alone is insufficient**: Cycle 0001's geometry-only approach (wider buckets) failed despite reducing collisions; cycle 0005's push to 0.75 geometric factor overshot. Conversely, early cycles showed stash capacity scaling alone faced diminishing returns by cycle 0003. Success requires both collision reduction *and* recovery coverage, operating together.

## Open questions

- **Promotion-score sensitivity**: Cycle 0010's +5.0% breakthrough used frequency-weighted scoring with a specific formula (`cold_access_count / (1.0 + time_since_eviction_ms)`). How much of that gain depends on the exact weights or time-decay function? Do alternative formulas yield further refinement?

- **Composition with LRU within hot tier**: Cycle 0009 showed hot-tier LRU provided +0.5% gain over FIFO. Does this compose additively with cycle 0010's frequency-weighted promotion? Could the combination achieve cumulative improvements?

- **Multi-tier with frequency weighting**: Cycle 0010 v002 (three-tier with intermediate warm buffer) encountered a build failure. If fixed and combined with frequency-weighted promotion at both warm→hot and cold→warm transitions, could it outperform the two-tier + frequency-weighted design? Or does two-tier + intelligent prioritization already capture that value?

- **Saturation and remaining headroom**: After 10 cycles of optimization (geometry, capacity, organization, prioritization), have we approached the workload's natural ceiling, or do additional orthogonal axes (e.g., predictive prefetching, multi-level frequency aging, workload-phase-aware geometry adaptation) remain unexplored?

- **Borrow checker resolution**: Three cycles (0003, 0005, 0009) encountered Rust borrow checker constraints preventing stash capacity exploration beyond 25%. If resolved through refactoring, could capacity scaling combined with promotion prioritization yield further gains? Or is 25% + frequency-weighted prioritization already the saturation point?

## Per-policy notes

- **FIFO within stash**: Baseline eviction order in all cycles. When combined with asymmetric partitioning (cycles 0007–0009) and later with frequency-weighted promotion prioritization (cycle 0010), FIFO proved sufficient and outperformed LRU-ordering refinements at the same tier level. Simplicity is an advantage when the organization layer (two-tier structure, promotion rules) carries the adaptive signal.

- **LRU within hot tier**: Introduced cycle 0009, yielded +0.5% gain over FIFO when applied to hot-tier eviction decisions. Modest but consistent secondary lever capturing temporal recency of individual item access patterns. Benefit plateaus because the two-tier organization itself already captures coarser temporal patterns (re-accessed vs. not re-accessed).

- **Asymmetric FIFO partitioning**: Cycles 0007–0009 explored partition ratios (18/7, 20/5, 19/6) with FIFO eviction. Asymmetry (allocating more capacity to hot tier than cold) proved superior to symmetric splits, reflecting workload's skewed re-access distribution. Partitioning space appeared saturated by cycle 0009 (19/6 yielded no additional gain).

- **Frequency-weighted promotion prioritization**: Cycle 0010 breakthrough mechanism. Scores cold-tier items by access count and recency of eviction, then allocates scarce hot-tier capacity preferentially to high-scoring items. Addresses the real axis of optimization: *which* recovered items deserve premium space, not how many items or tiers exist. Represents shift from static allocation to per-item adaptive decision-making.
