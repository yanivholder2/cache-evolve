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
