# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-haiku-4-5-20251001_2026-06-06_05-09-07-908/cycles/0009

## Winners

- v003
- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-haiku-4-5-20251001_2026-06-06_05-09-07-908/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-haiku-4-5-20251001_2026-06-06_05-09-07-908/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-haiku-4-5-20251001_2026-06-06_05-09-07-908/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-haiku-4-5-20251001_2026-06-06_05-09-07-908/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-haiku-4-5-20251001_2026-06-06_05-09-07-908/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-haiku-4-5-20251001_2026-06-06_05-09-07-908/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

This cycle tested three refinements aimed at optimizing the hit-rate/latency frontier: aggressive geometry (85/15) paired with adaptive threshold scaling, pure geometry at 80/20 as a midpoint within the saturation zone, and adaptive threshold formula refinement (divisor 3 instead of 4) on balanced baseline geometry. All three variants succeeded, but with divergent trade-offs. The clear standout is v002's 80/20 pure-geometry variant, which improved both hit-rate and get latency with only modest put-latency cost. v003's formula refinement proved that decreasing the threshold divisor from 4 to 3 on balanced geometry unlocks simultaneous gains across both hit-rate and latency. v001's attempt to compose aggressive geometry (85/15) with adaptive thresholding failed to reconcile the opposing forces: while promoting items more aggressively helped individual get operations, the combined effect overwhelmed the stash tier, regressing both hit-rate and put latency.

## Patterns

- **Optimal geometry lies in the 75/25–80/20 band, not further**: v002's 80/20 geometry delivers strong hit-rate improvement (7.5% gain) and exceptional get-latency reduction (37% improvement) with a manageable put-latency cost. This confirms that the hit-rate/latency frontier is well-calibrated in the mid-aggressive range, closer to the 75/25 incumbent than the 90/10 saturation boundary identified in cycle 0005. Pushing geometry further (v001's implicit pressure via aggressive threshold on 85/15) introduces unbalanced trade-offs; pulling back slightly (v002's 80/20) maximizes the favorable region.

- **Adaptive threshold formula sensitivity is tunable**: v003's shift from divisor 4 (cycle 0005) to divisor 3 demonstrates that formula constants directly control promotion selectivity and can be refined to shift the hit-rate/latency frontier. On baseline 25/75 geometry, threshold=8 (divisor 3) permits more candidates to promote than threshold=6 (divisor 4), simultaneously improving hit-rate and reducing latency. This suggests the prior formula over-filtered on balanced geometry, leaving hot items trapped in lower tiers. The formula's adaptation to stash bucket count makes these refinements portable across geometries.

- **Composition succeeds when it aligns rather than opposes geometry pressure**: v001 fails because 85/15 geometry (aggressive) paired with threshold=1 (also aggressive) creates compounding promotion pressure—more items flood the probing tier faster, overwhelming the 5-bucket stash and triggering eviction cascades. v003 succeeds because divisor 3 on balanced geometry aligns with the stash's capacity: 8 stash buckets on baseline can absorb the extra promotion candidates without churn. Composition is not orthogonal; it is constrained by the geometry's capacity envelope.

- **Simple geometry outperforms complex composition when both win**: Among the three winners, v002 (pure 80/20 geometry) achieves the cleanest outcome—no surprising trade-offs, no latency regressions, transparent hit-rate/latency coupling. v003 also succeeds, but via formula tuning on conservative geometry, leaving v001's composition approach as the least reliable. This reinforces the prior cycle's insight: when multiple strategies succeed, simpler mechanisms typically deliver more robust results.

- **Hit-rate degradation remains the principal cost of aggressive optimization**: v001's latency gains on get operations are negated by hit-rate regression and put-latency overhead. Both metrics together matter more than either in isolation; workloads reward capacity preservation over operation-level acceleration.

## Open questions

- Would adaptive threshold divisor 3 on v002's optimal 80/20 geometry combine the hit-rate and latency gains of both v002 and v003, or would the composition again encounter interference?
- Is there an even finer-tuned divisor (e.g., between 3 and 4) that optimizes the adaptive formula for mid-range geometries, or does divisor 3 represent a stable attractor?
- Can the formula divisor itself be made adaptive (scaled by geometry ratio) to achieve universal optimality across all allocations, or is each geometry band best served by a different constant?

## Continuation

The optimal geometry is now localized to the 75/25–80/20 range; stop exploring beyond 80/20. The key breakthrough in this cycle is that adaptive threshold formula refinement (divisor 3) unlocks clean composition by increasing promotion permissiveness on balanced geometry without incurring trade-offs. Next cycle should test whether divisor 3 composes with v002's 80/20 geometry to yield the best of both: hit-rate and latency gains from geometry plus gating refinement. If that composition succeeds, the optimization frontier has been pushed forward and future cycles can explore entirely new axes (e.g., eviction policies, bucket-size tuning, or tier-crossing heuristics beyond frequency). If composition at the optimal geometry still encounters interference, the evidence will be strong that simple geometry-only variants are the most reliable path forward and that frequency gating, despite its theoretical promise, has been fully optimized on this workload. Prioritize the divisor 3 + 80/20 composition test as the definitive next step.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

This cycle tested three points across the adaptive-threshold parameter space: aggressive 85/15 geometry with increased selectivity (divisor 2), optimal 80/20 geometry with moderate permissiveness (divisor 3), and baseline 25/75 geometry with maximum permissiveness (divisor 2). All three variants succeeded without regression—a watershed moment. Each composition of geometry and threshold divisor improves both hit-rate and latency. The standout is v002's 80/20 geometry with divisor 3, which maintains the clean orthogonality established in cycle 0006 while v001 and v003 extend the frontier in opposite directions (toward aggression and permissiveness, respectively).

## Patterns

- **Adaptive threshold scaling has fully matured across the allocation spectrum**: Unlike cycle 0006, where composition at one extreme failed, all three variants succeed. The framework now reliably aligns promotion selectivity with stash capacity across geometries ranging from baseline (25/75, 24 stash buckets) to aggressive (85/15, 5 stash buckets). This suggests the formula-based approach has solved the parameter coupling problem that plagued earlier cycles.

- **Threshold divisor forms an inverse relationship with geometry aggressiveness**: v001 requires divisor 2 (threshold=2) on tight 85/15 to filter false-positives; v002 thrives on divisor 3 (threshold=2 for 6 buckets) at the optimal 80/20 allocation; v003 permits divisor 2 (threshold=12) on spacious baseline 25/75. The tighter the stash capacity, the higher the divisor and stricter the gate. This inverse coupling explains why v001 failed in cycle 0006 (divisor 4 on 85/15 yielded threshold=1, too lenient) and succeeded here (divisor 2 yields threshold=2, properly calibrated).

- **Composition works reliably when parameters are inversely tuned**: v001's breakthrough is not new geometry or logic, but higher selectivity (threshold 1→2) on the same 85/15 allocation. This recovery of hit-rate—which was lost to cascading over-promotion in prior composition attempts—demonstrates that the interference pattern observed in cycle 0006 was due to parameter misalignment, not fundamental incompatibility between geometry and gating.

- **All winners preserve hit-rate while improving latency**: A crucial shift from earlier cycles. Each variant gains on at least one latency metric (get or put) without sacrificing hit-rate. This signals the optimization space has matured beyond the trade-off frontier and is entering a region where orthogonal improvements can coexist.

- **Simple pure-geometry variants remain competitive but no longer dominant**: v002's 80/20 with divisor 3 matches prior cycles' stand-alone geometry results, yet it is now joined by compositions that deliver equivalent or better outcomes. This suggests geometry alone is no longer the bottleneck; threshold tuning has become an equal partner in the optimization landscape.

## Open questions

- Among the three winners, which represents the best overall outcome? Is there a strict ordering (one dominates the others), or do they occupy a Pareto frontier with different trade-offs?
- Has the permissiveness frontier been reached (divisor 2 on baseline), or could divisor 3, 2.5, or lower yield further improvements?
- Can a meta-formula that scales the divisor based on both geometry ratio and stash bucket count achieve universal optimality, or is each geometry band best served by a fixed constant?

## Continuation

Adaptive threshold scaling is now a reliable and composable optimization mechanism across the entire tested allocation spectrum. The inverse relationship between geometry aggressiveness and threshold permissiveness is established empirically. All three winners represent successful tuning points, suggesting the optimization space is no longer constrained by parameter coupling. Next cycle should determine whether these three variants represent a Pareto frontier or whether one is strictly superior, and if so, adopt it as the new baseline to explore orthogonal optimization axes (eviction policies, bucket sizing, tier-crossing heuristics). If a Pareto frontier exists, stop refining the threshold formula and shift focus to entirely different mechanisms for further gains. Do not pursue further divisor refinement unless the next cycle reveals a clear new attractor.


### Cycle 0008

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
