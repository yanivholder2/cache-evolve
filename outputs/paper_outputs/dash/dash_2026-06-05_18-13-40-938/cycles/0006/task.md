# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0006

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0006/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
