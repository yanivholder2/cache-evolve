# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

This cycle tested three micro-optimizations to the LIRS-style tier hierarchy, all targeting the stash and its interactions with higher tiers. Two strategies improved hit rate by accelerating proven-hot items (stash residents) through the tier hierarchy; a third strategy testing frequency-based eviction showed no gain, confirming recency dominates this workload.

## Patterns

- **Stash hits merit fast-tracking**: Both hit-rate improvements (v001 and v003) prioritized stash-resident items for accelerated promotion, treating them as strong signals of interest worthy of immediate re-elevation toward the hot tier.
- **Positioning within a tier is a meaningful lever**: v001 achieved gains by changing insertion position from front (cold) to back (MRU) within the same target tier, suggesting that query order within a tier meaningfully impacts promotion eligibility.
- **Tier bypass trades traversal cost for placement priority**: v003 skipped the target tier entirely for stash hits, implying that elimination of intermediate steps can outweigh the benefit of tier-by-tier filtering.
- **Frequency-based eviction underperforms on recency-dominated workloads**: v002's LFU policy yielded no improvement despite correct implementation, validating that temporal locality (recent = reused soon) is a stronger predictor than access count on this traffic pattern.

## Open questions

- Why was v001 selected as the winner over v003, which also improved hit rate? (Magnitude of gain, tail behavior, or interaction with other subsystems?)
- Do v001 and v003 create synergy if combined, or do they interfere?
- Is there a sweet spot between fast-tracking and tier-by-tier filtering for stash hits?

## Continuation

Focus the next cycle on deepening stash-hit acceleration: both winning ideas converged on this axis, suggesting room for further refinement. Drop frequency-based eviction policies; this workload's recency-heavy access pattern makes them misaligned. Explore whether combining tier-position optimization (v001's lesson) with tier-skipping (v003's lesson) yields better results, or test variations on how stash-hitting items are weighted in promotion candidate selection.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cycle tested three strategies for accelerating stash-hit promotion through the tier hierarchy. The winner (v001) validated that combining direct tier-skipping with MRU positioning yields measurable hit-rate improvement; a second successful variant (v002) showed that adaptive occupancy-based strategies can also improve hit rate. A third strategy attempting unconditional backpressure relief via occupancy thresholds hurt hit rate by promoting non-hot items, confirming that occupancy alone is not a sufficient signal for acceleration.

## Patterns

- **Fitness signals gate acceleration**: Both successful variants preserved item hotness or recency signals before bypassing tiers. v001 promoted only stash hits (proven survivors); v002 also promoted only stash hits, adding an occupancy check. v003's failure—promoting any target item when stash > 60%—showed that occupancy-driven relief without a fitness check creates churn rather than relief.
- **Threshold sensitivity in occupancy-adaptive strategies**: v002's 75% occupancy threshold on stash-hit promotion succeeded, while v003's 60% threshold on unconditioned target promotion failed. The threshold alone doesn't determine success; the fitness check (stash hit vs. target hit) is more critical than the occupancy level.
- **Two paths to stash-hit acceleration converged**: v001 (deterministic tier-skip) and v002 (occupancy-adaptive tier-skip) both work, suggesting that the core insight—fast-tracking stash hits to probing—is robust across at least these two implementation strategies.
- **MRU positioning remains relevant**: v001's explicit use of Back insertion for MRU placement validates the cycle 0001 lesson that position within a tier matters; v002 implicitly inherited this by copying v001's logic.

## Open questions

- Why was v001 selected as winner over v002, despite both improving hit rate? (Simplicity, consistency, magnitude of gain, or tail behavior?)
- Does v001's deterministic acceleration degrade under sustained high stash occupancy, where v002's adaptive switch might excel?
- Is there an occupancy threshold where adaptive strategies (like v002) outperform deterministic ones (like v001), or are they equivalent under this workload?
- Could v002's adaptive logic be applied to v001 (occupancy-aware MRU positioning) to preserve determinism while handling load spikes?

## Continuation

Keep v001 as the new baseline—it's simple, validated, and improves hit rate. Investigate whether v002 handles high-pressure scenarios better by testing both strategies under sustained cache congestion; if v002 wins at peak load, consider an adaptive hybrid for production. Definitively drop occupancy-as-a-relief-valve approaches unless they are gated by hotness checks; the v003 failure confirms occupancy is not a fitness signal. In the next cycle, focus on understanding whether the deterministic v001 or adaptive v002 generalizes better, and whether combining both (occupancy-aware stash-hit acceleration) would amplify gains.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

This cycle tested three refinements to MRU positioning and adaptive destination selection within the tier hierarchy. The winner (v003) validated that extending MRU back insertion to demoted items improves hit rate; a second successful variant (v002) showed the same benefit for target-tier promotions. A third strategy attempting occupancy-adaptive destination selection alongside back insertion yielded no improvement, confirming that consistent positioning rules outperform conditional branching.

## Patterns

- **MRU positioning is a universal lever across all tier transitions**: Both successful variants (v002 and v003) applied back insertion consistently. v002 extended it from stash-hit promotions to target-tier advancements; v003 applied it to the demotion cascade. The cohort's success converges on this single mechanism—not promotion vs. demotion strategy, but insertion position.
- **Items with tier advancement history deserve MRU treatment**: v003's insight is that demoted items—those that previously climbed the hierarchy—benefit from immediate re-promotion eligibility upon reinsertion. This mirrors v002's lesson that target-tier items surviving to promotion deserve MRU positioning. The pattern suggests the system should reward proven items, not penalize them.
- **Occupancy-adaptive branching adds complexity without gain**: v001 combined adaptive destination selection with back insertion, yet hurt hit rate compared to the deterministic baseline. This suggests that occupancy sensing in destination choice is noise; the core gain comes from consistent, deterministic MRU positioning regardless of load state.
- **Tier-by-tier progression + consistent positioning works better than conditional tier-skipping under variable load**: Unlike cycle 0002's ambiguity between deterministic skip (v001) and adaptive destination (v002), cycle 0003 clarifies that the demotion path benefits from simplicity: always insert demoted items at back, always prioritize tier-advancing items consistently.

## Open questions

- Do v002 and v003 combine additively (both applied together) or do they saturate a single underlying effect?
- Why was v003 selected as winner over v002 when both helped hit rate? (Is demotion path optimization more impactful than promotion path, or is it a magnitude/consistency question?)
- Does the universal MRU principle extend to other insertion sites (e.g., fresh evictions in intra-tier operations, or cascade paths not yet tested)?

## Continuation

Keep v003 as the new baseline—it validated that demotion cascades benefit from the same MRU philosophy proven in earlier promotion work. Strongly consider testing v002 and v003 stacked together in the next cycle to measure synergy. Abandon occupancy-adaptive destination selection; the cycle 0003 result confirms that consistent, deterministic positioning rules are more robust than conditional logic gating tier selection. Focus next cycle on universalizing MRU back insertion across remaining unexamined paths (intra-tier operations, secondary promotion checks) and testing whether the combination of v002 + v003 + deterministic skip-to-probing creates the strongest overall strategy. The cohort pattern suggests the system converges on simple, uniform positioning rules rather than complex conditional branching.


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

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
