# Cycle 0008 Insight

## Cohort outcome

This cycle tested three distinct tuning strategies on the established threshold-geometry framework: stricter divisor 2 gating on optimal 80/20 geometry, a geometry-aware adaptive divisor formula that scales based on allocation ratio, and more permissive divisor 4 gating on baseline 25/75 geometry. All three variants succeeded. The standout is v002's adaptive divisor formula, which decouples operation latency from hit-rate by adjusting promotion selectivity based on the normal-to-stash bucket ratio itself—a meta-parametrization that promises geometry-independent composition. v001 and v003 succeed via opposing tuning directions (stricter and more permissive, respectively), suggesting the optimization space now contains multiple stable tuning points.

## Patterns

- **Threshold divisor is maturing from geometry-specific constant to geometry-aware formula**: Prior cycles tested fixed divisors (2, 3, 4) paired with specific geometries. This cycle's breakthrough is H02's proposal to make the divisor itself adaptive—computed from allocation ratio rather than hand-tuned per geometry. v002's early implementation of this idea achieves dramatic latency wins (57% on get p50) while holding hit-rate flat, suggesting the formula can decouple computational cost from selectivity. This elevates threshold tuning from a constant-selection problem to a parametric meta-formula problem.

- **Multiple opposing tuning directions now succeed without regression**: v001 (divisor 2, stricter) and v003 (divisor 4, more permissive) both win while maintaining the hit-rate plateau around 0.7166–0.7160. This contrasts sharply with cycle 0006, where composition attempts at extremes failed. It suggests the optimization space has matured sufficiently that selectivity can be tuned across a range without triggering cascading failures—evidence that adaptive threshold scaling has truly decoupled promotion logic from geometry pressure.

- **Divisor selectivity and geometry aggressiveness remain inversely coupled, but now as a design principle**: v001's success on aggressive 80/20 with stricter divisor 2, paired with v003's success on conservative 25/75 with looser divisor 4, reinforces the inverse relationship discovered in cycle 0007. More permissive divisor on baseline geometry enables hot items to promote without unreachable thresholds; stricter divisor on tight geometry filters false-positives that would overwhelm stash capacity. This inverse coupling is not an artifact to be overcome—it is the foundation on which reliable composition rests.

- **Adaptive formula based on allocation ratio (H02) represents the next frontier**: Unlike prior cycles' fixed-divisor approaches, H02's meta-formula adjusts divisor based on the geometry's normal-to-stash ratio. This single concept—making the divisor itself adaptive—positions threshold tuning to become truly orthogonal to geometry selection. If successful, it eliminates the fragile coupling that plagued earlier cycles and enables future work to explore entirely new axes without re-tuning thresholds.

- **Latency improvements now come from computational efficiency, not just selectivity**: v002's massive get-latency reduction (57% on p50) occurs without hit-rate change (0.6634 flat), suggesting the gains come from improved memory patterns or compiler optimization enabled by the adaptive formula, not from changing what gets promoted. This decoupling of latency from selectivity is conceptually important: it means threshold tuning can address two independent concerns—what to promote (selectivity) and how efficiently to compute the decision (formula structure).

## Open questions

- Does H02's geometry-aware adaptive divisor formula succeed across the full allocation spectrum (baseline, mid-aggressive, aggressive), or does it encounter interference at certain ratios?
- Is a two-parameter meta-formula (stash buckets + allocation ratio) sufficient for universal optimality, or does the divisor computation require additional factors (e.g., workload skewness)?
- Among the three winners, which represents the best overall outcome? Does v001's strict tuning on optimal geometry outperform v002's latency-optimized formula, or does v003's permissiveness on baseline open a competitive path?

## Continuation

Divisor-based threshold scaling has matured into a meta-parametrization problem: the divisor itself should be adaptive to allocation ratio, not fixed per geometry. H02's geometry-aware adaptive divisor is the most strategic avenue and should be prioritized in the next cycle. If the meta-formula succeeds across all allocation ratios (capturing the inverse selectivity-geometry relationship automatically), threshold tuning is effectively solved and future cycles should abandon further divisor refinement in favor of entirely new optimization axes (eviction policies, tier-crossing heuristics, bucket sizing). If the meta-formula encounters interference or fails to unify the behavior of v001, v002, and v003, revert to v001's conservative stricter-divisor approach on optimal 80/20 geometry and freeze threshold exploration. Do not pursue permissiveness further (v003's direction); the evidence from v001 and v002 strongly suggests selectivity is the limiting constraint on this workload.
