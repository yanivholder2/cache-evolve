# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0009

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

This cycle tested three fundamental architectural levers after five cycles of insertion-mechanics refinement: stash capacity rebalancing (v001), bucket allocation skewing (v002), and tier-skip revival (v003). Only v002 achieved improvement over the 0.684 baseline with a modest +0.0014 hit-rate gain by rebalancing normal buckets toward the probing tier. v001's stash expansion and v003's tier-skip both hurt hit-rate, confirming that capacity is a secondary lever and that tier-skip remains fundamentally incompatible with demotion-back-insertion discipline.

## Patterns

- **Probing-tier capacity is a real but marginal bottleneck**: v002's +0.0014 improvement from skewing bucket allocation (18 probing vs. 10 target) demonstrates that hot-tier residents benefit from expanded capacity. However, the modest gain—achieved while shrinking target capacity from 224 to 160 slots without measurable penalty—indicates that capacity reallocation is not a primary driver. The system has room for minor optimization through geometric tuning, but this lever alone cannot break through architectural ceilings.

- **Stash buffering expansion does not address the root constraint**: v001's failure confirms that extending the stash filtering window (4→6 buckets) doesn't improve hit-rate. The bottleneck is not insufficient time to characterize access patterns in the stash, but a deeper architectural factor in how items progress through tiers or how the probing/target interaction drives eviction policy.

- **Tier-skip remains architecturally incompatible with demotion-back-insertion**: v003's failure reiterates cycle 0004's finding: tier-skip (stash→probing bypass) conflicts with the demotion philosophy that rewards tier-advanced items with MRU insertion. Tier-skip floods the probing tier with unvetted stash hits that lack the tier-advancement proof that justifies MRU insertion, polluting the hot tier and degrading hit-rate. This incompatibility is structural, not experimental-noise.

- **The 0.684 ceiling is defended by architectural asymmetries, not geometric tuning**: Five cycles of insertion mechanics (0002–0005) and now three cycles of geometric/architectural testing (this cycle) all converge on a ~0.684 boundary. Insertion position, occupancy thresholds, stash buffering, and capacity skewing all fail to break through. This suggests the constraint is rooted in asymmetric tier interactions (e.g., how promotion discipline interacts with demotion recovery, or how target-tier filtering gates probing access) rather than in any single knob.

## Open questions

- Why does capacity skewing toward probing yield only marginal gain (+0.0014)? Is it because the workload's access distribution naturally suits the original 14:14 split, or because capacity is genuinely not a primary constraint?
- What architectural interaction is preventing demotion-back-insertion from stacking beneficially with tier-skip? Is it eviction cascade timing, probing-tier saturation, or MRU eligibility semantics?
- Is the 0.684 boundary truly optimal for this workload, or do untested levers (eviction ordering policies beyond LRU, stash depth, bucket saturation triggers) hold the key to further gains?

## Continuation

Accept that capacity and geometric tuning are marginal levers: v002's +0.0014 gain may represent the ceiling for this class of optimization. Abandon tier-skip as a viable path; it is fundamentally incompatible with demotion-back-insertion and no isolated testing will resolve that conflict. For the next cycle, abandon insertion mechanics and geometric tuning entirely and instead investigate the asymmetry between promotion and demotion paths: test whether alternative eviction orderings (e.g., evict from target before probing, or bucket-saturation-triggered cascades), different tier thresholds for advancement, or stash-depth variations can unlock headroom. If those levers also prove inert, consider whether 0.684 is the true optimum for this workload and system, and document the architectural constraints that defend it. Keep v002's capacity reallocation as a minor optimization additive to the v003 demotion-back-insertion baseline if next cycle's testing allows it.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
