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
