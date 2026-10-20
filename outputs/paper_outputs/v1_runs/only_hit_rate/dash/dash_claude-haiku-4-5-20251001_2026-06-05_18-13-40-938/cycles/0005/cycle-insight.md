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
