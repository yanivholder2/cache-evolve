# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-haiku-4-5-20251001_2026-06-06_05-09-07-908/cycles/0010

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-haiku-4-5-20251001_2026-06-06_05-09-07-908/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-haiku-4-5-20251001_2026-06-06_05-09-07-908/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-haiku-4-5-20251001_2026-06-06_05-09-07-908/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-haiku-4-5-20251001_2026-06-06_05-09-07-908/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-haiku-4-5-20251001_2026-06-06_05-09-07-908/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-haiku-4-5-20251001_2026-06-06_05-09-07-908/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

This cycle tested three distinct refinement strategies on the established geometry-and-threshold framework: further geometry fine-tuning toward stash (79/21 from the 80/20 optimum), a new orthogonal eviction policy axis using frequency-aware item ordering, and geometry-aware divisor co-optimization with larger stash capacity (75/25 with divisor 4). v001's geometry push failed, confirming the 80/20 optimum is locally stable. v002 and v003 both succeeded, but via divergent mechanisms: v002 through implementation efficiency (substantial latency gains without hit-rate cost) and v003 through parameter co-optimization (stricter divisor paired with larger stash capacity, preserving hit-rate while improving latency).

## Patterns

- **The 80/20 geometry is definitively locally optimal**: v001's single-percentage movement toward stash (79/21) degraded both hit-rate and latency compared to the 80/20 baseline. This confirms that the mid-range band identified in prior cycles (75/25–80/20) contains a sharp peak at 80/20 itself; movement in either direction incurs cost. The frontier is not further refinable by incremental geometry tuning.

- **Latency gains can be decoupled from hit-rate through two independent paths**: v002 demonstrates that implementation optimization—replacing vector-shuffling with in-place counters—delivers 57% faster get operations and 18% faster put operations without any hit-rate change. v003 demonstrates that geometry-aware parameter co-optimization (larger stash capacity paired with stricter divisor gating) delivers similar latency improvements while preserving hit-rate. These are mechanically orthogonal solutions to the same latency constraint.

- **The adaptive divisor formula continues to exhibit inverse coupling with geometry aggressiveness**: v003's success on 75/25 geometry with stricter divisor 4 confirms and extends the pattern from cycle 0007–0008: more stash capacity permits and requires stricter gating to avoid overwhelming the tier. The formula is not geometry-independent; it is geometry-aware and composition-sensitive.

- **Frequency information does not improve decision quality on this workload**: v002's key insight is that frequency-aware eviction adds implementation complexity without providing additional cache decision advantage. Recency and frequency are well-correlated on workload-a, making pure LRU sufficient. This closes an exploratory branch: frequency-biased eviction is a dead-end for this access pattern.

- **The optimization space is converging on multi-factor improvements**: Unlike cycle 0006–0007, where composition attempts often failed, both v002 and v003 succeed without regression. All three winners maintain or improve latency; v001's failure is not a regression but a confirmation of boundary conditions. This signals the optimization space has matured sufficiently to support independent improvements on orthogonal axes.

## Open questions

- Is v001's failure purely due to geometry, or does the divisor choice (2 on 79/21) also contribute? Could a different divisor on 79/21 succeed?
- Among v002 (efficiency) and v003 (geometry-formula), which represents the more sustainable optimization path? Is there a strict ordering, or do they occupy a Pareto frontier?
- Can v002's implementation efficiency and v003's geometry-aware formula be combined, or do they compete for the same latency-improvement budget?

## Continuation

The geometry frontier is now tightly bounded and locally stable at 80/20; stop pursuing incremental geometry refinement. Frequency-aware eviction is confirmed ineffective on this workload—abandon this axis. The two competing paths forward are (1) pure implementation optimization for latency (v002) and (2) geometry-aware parameter co-optimization (v003). Both succeed without hit-rate cost, meaning the optimization space now permits orthogonal gains. Next cycle should test whether these two approaches compose: can v002's efficient eviction implementation be applied to v003's 75/25 geometry with adaptive divisor to combine latency and hit-rate improvements, or does their interaction reveal interference? If composition succeeds, the result becomes the new optimized baseline. If not, choose the variant with the strongest overall outcome (hit-rate + latency balance) as the new standard and shift exploration entirely away from geometry and thresholding toward new axes (e.g., tier-crossing heuristics, promotion cascading policies, or workload-specific tiering strategies).


## Write

Exactly one file: `cycle-insight.md` in the output dir.
