# Cycle 0009 Insight

## Cohort outcome

This cycle consolidated the architectural breakthrough from cycle 0008 by testing whether the unified two-tier hierarchy (Stash + unified Hot) could be enhanced with conditional acceleration logic. v001 confirmed that the simplified two-tier architecture remains the winner, improving hit-rate by combining tier simplification with direct stash-to-hot promotion and occupancy-aware back-insertion demotion. v002 and v003 both failed to improve on this baseline, demonstrating that additional conditional logic (occupancy-gated tier-skip, stricter promotion thresholds) adds complexity without compensating gains.

## Patterns

- **Simple, direct promotion paths outperform gated acceleration**: v002's occupancy-gated tier-skip did not improve hit-rate over the baseline two-tier architecture, despite the hypothesis that load-aware acceleration would unlock further gains. This indicates that the simple stash-to-hot direct path (without gating) is already near-optimal for this workload's access pattern; congestion-sensitive routing adds implementation overhead without discovering latent cache utility.

- **Promotion selectivity has a sweet-spot threshold that tightening violates**: v003's degradation from raising the access_count threshold (2→3) reveals that filtering discipline carries a cost in promotion latency. Items with 2+ accesses in stash still generate value in the hot tier, but denying them access longer forces unnecessary re-qualification paths. The ≥2 threshold inherited from cycle 0008's gated tier-skip remains better balanced than stricter selectivity for this workload.

- **Architectural simplification proved more important than routing intelligence**: Across cycles 0002–0007, occupancy-aware routing, eviction priority, cascade structure, and capacity tuning all failed to exceed 0.684. Cycle 0008's breakthrough came from eliminating the intermediate tier filtering entirely (target tier gone), not from adding occupancy-aware logic. v001 carries forward the unified two-tier structure and achieves its best results, while v002's attempt to add conditional routing on top of that structure yields flat performance. The bottleneck was the three-tier topology itself, not the lack of load awareness in routing.

- **The two-tier unified hot tier represents an architectural equilibrium for this workload**: v001's win, v002's flat result (no degradation), and v003's failure converge on a pattern: once the three-tier hierarchy is eliminated and direct promotion is enabled, further tuning via conditional logic or stricter filtering does not improve hit-rate. The architecture has moved from a parametric plateau (0.684 defended by insertion position, occupancy, eviction) to an architectural equilibrium (two-tier, direct promotion, MRU demotion back-insertion) that is robust to overlay mechanisms but inert to additional tuning.

## Open questions

- What is the magnitude of v001's improvement over the cycle 0008 baseline? Understanding whether the gain is marginal (0.001–0.005) or substantial (>0.01) clarifies whether further architectural exploration is warranted or whether consolidation is approaching diminishing returns.

- Can the two-tier unified architecture be further simplified (single tier?), or are there other orthogonal architectural levers (alternative eviction orderings, bucket saturation triggers, tier-count variations) that could unlock additional headroom?

- Does the workload's natural access distribution inherently suit a two-tier direct-promotion model, or is the architecture's simplicity the primary driver of improvement?

## Continuation

Consolidate the two-tier unified architecture as the new baseline: v001's architecture (Stash + unified Hot with all 28 buckets allocated to Hot, direct stash-to-hot promotion, MRU back-insertion demotion) is now the proven optimum for this workload. Abandon further conditional logic (occupancy-gating, stricter thresholds) on top of this structure—v002 and v003 confirm it adds complexity without marginal gains. For the next cycle, if continued improvement is desired, pivot toward orthogonal architectural questions: test whether alternative eviction orderings (non-LRU), different tier counts, or bucket-saturation-triggered cascades can unlock additional headroom, or whether the two-tier equilibrium with direct promotion represents the natural optimum for this cache size and workload profile. Document v001 as the current champion and clarify whether the goal is further optimization or architectural documentation.
