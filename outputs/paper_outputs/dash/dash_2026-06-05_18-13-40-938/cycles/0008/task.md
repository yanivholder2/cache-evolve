# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0008

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

This cycle tested three distinct levers to break through the 0.684 hit-rate ceiling from cycle 0004: insertion position asymmetry, promotion-eligibility gating, and occupancy-based filtering. All three variants failed to improve upon baseline, with v003 dropping below it. The cohort reveals that the local optimum is robustly defended against refinements to insertion mechanics, candidacy timing, and occupancy thresholds alike.

## Patterns

- **Insertion position is not the bottleneck for promotion paths**: v001's reversion of promotion paths to Front insertion (LRU) while preserving demotion back insertion held performance steady at 0.684. This confirms cycle 0004's hypothesis-questioning observation: the asymmetry problem lies deeper than where items land upon arrival. Promotion-path positioning—whether Front or Back—does not determine hit-rate trajectory.

- **Candidacy gating alone cannot unlock further gains**: v002's eligibility-delay mechanism (requiring newly promoted items to prove fitness before advancing again) maintained baseline performance without improvement. This indicates the bottleneck is not *how quickly* items become eligible for promotion, but rather *which items* should be promoted and *how tier progression* interacts with eviction pressure. Delaying candidacy leaves the underlying tier interaction unchanged.

- **Occupancy thresholds continue to suppress rather than improve hit rate**: v003's occupancy-gated promotion (70% threshold) reduced hit rate below baseline, reinforcing cycle 0004's lesson that occupancy-based filtering without a deeper fitness signal promotes suppression, not discipline. Deferring advancement of stash-hit items during high occupancy appears to prevent timely identification of genuinely hot items rather than improve filtering.

- **The 0.684 baseline is a robust equilibrium, not a temporary plateau**: Three orthogonal approaches—insertion position, promotion cadence, and occupancy-aware gating—all failed to exceed it, with only one (v001) matching it. This suggests the ceiling is enforced by architectural factors beyond these insertion and eligibility knobs (tier saturation dynamics, eviction policy interactions, bucket mechanics), not by tuning any single parameter.

- **Demotion recovery remains the stable foundation**: Despite testing very different hypotheses, the demotion back-insertion mechanism from cycle 0003 remained untouched in all variants and appears to be the only working mechanism carrying forward. This suggests the system has settled into an asymmetric equilibrium: demoted items benefit from MRU recovery, but fresh promotion paths do not benefit from further acceleration.

## Open questions

- Is 0.684 a true local optimum for this workload, or is there headroom unlocked by levers we haven't yet tested (tier capacity ratios, eviction ordering, stash depth, bucket saturation thresholds)?
- What architectural factor is constraining hit rate: is it eviction cadence, tier occupancy balance, bucket saturation, or a interaction effect between multiple components?
- Should we continue refining the current promotion-demotion asymmetry, or pivot to testing fundamentally different tier structures (e.g., different capacity ratios, alternative eviction policies)?

## Continuation

Abandon insertion position, eligibility gating, and occupancy-threshold refinements; this cycle confirms they have exhausted their leverage. The 0.684 baseline appears to be a local equilibrium enforced by architectural factors beyond positioning and timing knobs. For the next cycle, shift focus away from insertion mechanics entirely and investigate tier architecture and eviction policy: test whether different tier capacity ratios, alternative eviction orderings (beyond LRU/MRU), or stash saturation points unlock gains. If these levers also leave hit rate unchanged, consider whether 0.684 is the true maximum for this workload and the system should be accepted as optimized at its current configuration. Preserve demotion back insertion (v003's proven mechanism) as the baseline foundation for future testing.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
