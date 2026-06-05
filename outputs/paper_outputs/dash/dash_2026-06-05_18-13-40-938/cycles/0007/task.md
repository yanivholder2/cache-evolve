# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

This cycle tested three combinations of previously successful strategies (stash-hit acceleration, MRU positioning, tier skipping) and their interactions with the current best baseline. All three variants hurt hit rate compared to cycle 0003's winner, revealing hard boundaries in how these mechanisms compose. The key finding is that strategies succeeding in isolation can interfere when combined, suggesting the current baseline has reached a local optimum constrained by asymmetric rules across promotion and demotion paths.

## Patterns

- **MRU positioning has asymmetric scope**: v001 demonstrated that MRU back insertion works for demoted items (with proven tier-advancement history) but destabilizes newly promoted items in higher tiers. Rewarding demotion recovery differs fundamentally from rewarding fresh promotion; demoted items earn their MRU status by surviving tier discipline, while freshly promoted items still need to earn residency at a higher tier.
- **Intra-tier optimization is orthogonal to cross-tier progression**: v002 showed that repositioning accessed items within the stash tier—keeping them MRU-ranked for next access—does not translate to hit-rate improvement. The MRU principle drives gains only when coupled with tier advancement, not with ordering within a static tier.
- **Tier-skip and back-insertion strategies have conflicting assumptions**: v003 revealed that combining deterministic stash-hit tier-skip (which requires items to be heavily filtered by intermediate tiers before promotion) with back-insertion demotion (which restores demoted items to MRU in lower tiers) creates two incompatible classes of probing residents: earned climbers and fast-tracked re-entries. The tier-skipping mechanism assumes a filtering discipline that back-insertion undermines by giving demoted items a shortcut.
- **Simple, singular strategies outperform composite approaches on this workload**: Cycles 0001–0003 each succeeded with one clear mechanism (stash-hit promotion, MRU positioning on specific paths, demotion recovery). Cycle 0004's failures suggest these gains don't stack; the current best configuration (demotion back insertion alone) represents a balanced equilibrium rather than a starting point for further combination.

## Open questions

- Is the cycle 0003 baseline (v003, hit rate 0.6839) a local optimum, or is there headroom beyond the demotion + baseline approach?
- What is the precise mechanism that makes MRU insertion beneficial for demoted items but harmful for freshly promoted items? Is it eviction cadence, promotion candidacy eligibility, or something else?
- Are there completely different levers (e.g., tier capacity, stash eviction policy, bucket structure) that could improve hit rate without relying on the demotion-promotion asymmetry?

## Continuation

Stop pursuing combinations of previously successful strategies; this cycle confirms they have reached interaction limits. Instead, investigate the root asymmetry: why does the same MRU mechanism benefit demotion recovery but hurt fresh promotion? Consider testing variations on this asymmetry (e.g., position-based promotion eligibility delays, or demotion-specific insertion policies that don't apply to forward promotion) to see if you can unblock further gains. If the cycle 0003 baseline proves to be a local optimum, pivot to testing fundamentally different levers (eviction policies, tier sizing, stash capacity thresholds) rather than refining insertion position logic. Abandon intra-tier optimization; it is not a hit-rate driver.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
