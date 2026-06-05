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
