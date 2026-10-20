# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0005

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0005/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0005/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0005/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0005/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0005/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
