# Cycle 0007 Insight

## Cohort outcome

This cycle tested three orthogonal refinements to eviction and capacity mechanics after six prior cycles of insertion, positioning, and architectural tuning all converged on a 0.684 hit-rate ceiling: target-first eviction priority to protect hot-tier residents, cascade simplification by bypassing the target-tier retry step, and stash capacity expansion to improve filtering signal. All three variants either maintained the baseline (v001) or degraded it (v002, v003), demonstrating that the 0.684 boundary is now defended against mechanical refinement across eviction policy, demotion cascade design, and tier allocation alike.

## Patterns

- **Eviction priority shifting does not unlock further gains**: v001's target-first eviction (proactively demoting from target when probing fills) maintained baseline performance without improvement. This indicates that the ceiling is not constrained by *where* evicted items flow or which tier bears eviction burden, but by a more fundamental factor in how items arrive, navigate, and compete for promotion through the tier hierarchy.

- **Demotion cascades embody a necessary recovery principle, not a bottleneck**: v002's degradation (~3% hit-rate drop) when eliminating the target-tier retry step for foreign residents reveals that the "one-more-chance" safety valve is load-bearing. Items that land in probing as foreign residents (hash collisions, overflow) depend on a second chance to prove value in target before final demotion to stash. Removing this step prematurely routes borderline-useful items out of the promotion pipeline, confirming that the cascade structure is well-balanced, not over-engineered.

- **Stash filtering window is at capacity for this workload**: v003's degradation from expanding stash buckets (4→6) at the expense of normal buckets confirms cycle 0006's finding: the baseline allocation is tuned to the workload's access distribution. Extending the stash's characterization window does not improve hotness detection enough to justify shrinking the promotion tiers' resident capacity.

- **Seven cycles of orthogonal testing converge on 0.684 as a robust architectural equilibrium**: Across cycles 0002–0007, testing has spanned insertion position (Front/Back/MRU), promotion candidacy timing, occupancy-based gating, tier capacity ratios, tier-skip revival, and now eviction priority and cascade structure. None have exceeded 0.684 by more than the marginal +0.0014 from cycle 0006's capacity skew. The plateau is not a local optimum awaiting one more tuning—it is systematically defended across independent levers, indicating the bottleneck is architectural (how tiers interact and how items flow between them) rather than parametric (where items land, when they advance, how much space they occupy).

## Open questions

- Is 0.684 the true optimum for this workload's access pattern under a tier-based architecture, or is there a qualitatively different architectural pathway (e.g., non-LRU eviction ordering, alternative tier counts, bucket saturation triggers) that could unlock headroom?

- What specific interaction between promotion flow, eviction cascade timing, and tier occupancy is defending the 0.684 ceiling? Does the bottleneck lie in how stash→target graduation is gated, or in how target→probing advancement is determined?

- Should the next cycle abandon parametric tuning entirely and test whether the workload's inherent access characteristics naturally suit a 0.684 ceiling, making further gains infeasible with the current architecture?

## Continuation

The mechanical refinement phase is complete: eviction priority, cascade structure, and stash allocation have all proven inert or counterproductive. v001's maintenance of baseline (rather than improvement) confirms that shifting eviction burden between tiers is not a hit-rate driver. For the next cycle, abandon further tuning of existing mechanisms and instead test whether the bottleneck is structural rather than parametric: consider testing alternative tier counts, different promotion qualification criteria beyond simple LRU/MRU rules, or whether the workload's natural access distribution inherently suits a 0.684 ceiling. If these structural tests also fail to yield improvement, document 0.684 as the achieved optimum for this workload with the current tier-based cache design, noting that the ceiling is defended across insertion, positioning, occupancy, eviction, cascade, and capacity levers alike.
