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
