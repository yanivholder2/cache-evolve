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
