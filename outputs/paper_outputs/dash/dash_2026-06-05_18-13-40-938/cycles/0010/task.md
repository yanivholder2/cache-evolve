# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0010

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

This cycle marks a decisive pivot from seven cycles of parametric refinement to architectural restructuring. After occupancy-aware demotion routing (v001) failed once more to exceed the 0.684 baseline, the cohort tested two orthogonal architectural changes: collapsing the three-tier hot-side to a unified hot tier (v002) and gating tier-skip to proven stash residents with mandatory recovery enforcement (v003). Both v002 and v003 improved hit rate, breaking through the 0.684 ceiling for the first time in four cycles and validating that the bottleneck was architectural structure, not parametric tuning.

## Patterns

- **The three-tier hot-side hierarchy was the actual bottleneck**: v002's collapse of target and probing into a single hot tier, eliminating the intermediate gatekeeping step, improved hit rate. This directly contradicts the assumption from prior cycles that tier positioning and insertion mechanics were the constraints. The workload's access pattern tolerates—and benefits from—direct stash-to-hot promotion, suggesting the two-stage target→probing filtering was adding overhead without sufficient discriminatory signal for this workload.

- **Architectural simplification succeeds where parametric tuning failed**: Seven prior cycles tested insertion position, occupancy thresholds, eviction priority, cascade structure, and capacity allocation without exceeding 0.684. Cycle 0008's gains emerge only from changing the tier structure itself, not from tuning insertion position, eligibility gates, or occupancy routing. This confirms that the previous ceiling was enforced by the architecture's topology, not by any single parameter.

- **Tier-skip is viable when recovery is structured into the skip decision**: v003's conditional tier-skip (gated to stash residents with access_count ≥ 2, with skip_history_flag enforcing target re-entry on eviction) improved hit rate, directly refuting cycles 0004 and 0006's findings that tier-skip was "fundamentally incompatible" with demotion recovery. The key difference is that v003 eliminates the "unvetted hot-tier residents" problem by making recovery automatic: skipped items do not arrive at probing as unchecked stash hits, but as pre-qualified candidates who get a safety-check second chance in target before demotion. This reconciles two previously antagonistic strategies.

- **Occupancy-aware routing remains inert as a standalone lever**: v001's occupancy-coordinated demotion (bypassing target when >70% full) maintained baseline without improvement. This extends the pattern that occupancy-based gating does not improve hit rate. The findings suggest that occupancy awareness alone, decoupled from structural changes, is not a constraint; the leverage is in *how tiers are connected* (v002) and *how skip recovery is enforced* (v003), not in *occupancy triggers* (v001).

## Open questions

- Do v002 and v003 achieve equivalent hit-rate improvements, or does one exceed the other? Understanding the magnitude difference clarifies whether simpler hierarchy or structured tier-skip is more effective for this workload.

- Can v002's two-tier simplification and v003's gated tier-skip be combined into a single variant? A two-tier hierarchy with conditional tier-skip recovery might compound the gains if they are orthogonal.

- What is the improvement magnitude relative to prior cycles' baseline? Is this a modest (+0.001–0.005) or substantial (>0.01) breakthrough?

## Continuation

Architectural restructuring is now confirmed as the path forward; parametric tuning has been exhausted. For the next cycle, abandon occupancy-based routing entirely and focus on consolidating the architectural gains: test whether v002's simpler hierarchy and v003's gated tier-skip can be merged, since both improve hit rate through different mechanisms (eliminating gatekeeping overhead vs. reconciling acceleration with recovery discipline). If both are improvements, a combined variant merging two-tier hierarchy with conditional tier-skip recovery should be the next target. If they cannot be safely combined, document the trade-off and select the variant with the larger gain as the new baseline. Regardless, the 0.684 ceiling has been broken; document that the three-tier hot-side hierarchy and its two-stage filtering requirement were the root architectural bottleneck defended across all prior parametric tuning attempts.


### Cycle 0009

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
