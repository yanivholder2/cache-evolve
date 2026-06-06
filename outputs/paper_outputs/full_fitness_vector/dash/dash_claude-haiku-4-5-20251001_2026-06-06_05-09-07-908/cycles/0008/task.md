# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-haiku-4-5-20251001_2026-06-06_05-09-07-908/cycles/0008

## Winners

- v001
- v003
- v002

## Variant insights

- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-haiku-4-5-20251001_2026-06-06_05-09-07-908/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-haiku-4-5-20251001_2026-06-06_05-09-07-908/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-haiku-4-5-20251001_2026-06-06_05-09-07-908/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-haiku-4-5-20251001_2026-06-06_05-09-07-908/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-haiku-4-5-20251001_2026-06-06_05-09-07-908/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-haiku-4-5-20251001_2026-06-06_05-09-07-908/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

This cycle tested three refinements: lowering frequency-gating threshold to 1 on concentrated geometry, pushing geometry allocation to its asymptote (90/10), and implementing adaptive threshold scaling based on stash bucket count. Geometry concentration finally saturated at 90/10, revealing the boundary where hit-rate losses outweigh latency gains. Frequency-gating threshold tuning on fixed geometry proved counterproductive, but adaptive threshold computation succeeded, establishing a path toward geometry-independent gating that composes robustly across allocations.

## Patterns

- **Geometry asymptote identified at ~85/15–90/10**: The four-cycle progression of sustained improvement (25/75 → 50/50 → 62.5/37.5 → 75/25 → 85/15 → 90/10) finally encounters saturation. v002's 90/10 geometry improves get and put latency but regresses hit-rate as stash capacity shrinks to 3 buckets—insufficient for the Zipfian workload's tail. This definitively closes the question: geometry concentration is not a bottomless lever. The sweet-spot appears to be around 75/25–85/15, where both metrics improve; pushing past 85/15 optimizes for latency at the expense of hit-rate.

- **Hardcoded threshold tuning is geometry-specific, not universal**: v001's attempt to lower the frequency gate from threshold=2 to threshold=1 on 75/25 geometry demonstrates that the prior cycle's selectivity (threshold=2 on concentrated geometry) was already near-optimal. Reducing it further floods the 8-bucket probing tier with false-positives, causing collision chains and hit-rate dilution. The threshold is not a knob to turn independently; it is tightly coupled to the stash tier's capacity.

- **Adaptive threshold computation unlocks cross-geometry composition**: v003's breakthrough is shifting from a hardcoded constant (`PROMOTION_THRESHOLD: u16 = 2`) to a geometry-aware initialization parameter (`max(1, stash_buckets / 4)`). This single change eliminates the sensitivity to allocation profile: on 25/75 (24 stash buckets), the threshold becomes 6; on 75/25 (8 stash), it remains 2. By scaling the gate's selectivity with the stash size, v003 achieves 42–43% get-latency improvement and 9–19% put-latency improvement while holding hit-rate stable—proving that gating can become orthogonal to geometry when parameters adapt.

- **Simple adaptive logic beats complex per-geometry tuning**: v003 demonstrates that a single geometric scaling formula outperforms cycles of manual threshold calibration across different allocations. This shift from hardcoded constants to initialization-time parameter computation is mechanically trivial but strategically powerful: it removes the human coupling between geometry and promotion logic, enabling reliable composition.

- **Hit-rate remains the harder metric to preserve**: Across all five cycles, hit-rate degradation appears whenever either geometry or gating pushes too far (e.g., 90/10 or threshold=1). Latency improvements alone (v002's get/put gains) do not compensate for hit-rate regression on Zipfian workloads, confirming the prior insight that capacity utilization is more valuable than operation-level speed on skewed access patterns.

## Open questions

- Does adaptive threshold on aggressive geometry (e.g., 85/15 or a sweet-spot near 80/20) deliver better composition than v003's baseline results, potentially reclaiming some of the hit-rate margin?
- Is there an optimal geometry between 75/25 and 90/10 (e.g., 80/20 or 82/18) that maximizes both hit-rate and latency?
- Can the adaptive scaling formula itself be tuned (e.g., `stash_buckets / 3` or `stash_buckets / 5`) to shift the trade-off frontier, or is division-by-4 near-universal?

## Continuation

Adaptive parameter scaling is the key insight: frequency-gating selectivity must track geometry allocation to remain orthogonal, and this can be achieved via a simple initialization-time computation rather than per-operation overhead. Next cycle should test adaptive threshold on aggressive geometries (85/15 or a candidate sweet-spot like 80/20) to determine whether v003's composition success generalizes and to identify whether a geometry better than 75/25 exists. The geometry asymptote is now localized to the 85/15–90/10 range; stop pushing geometry further and focus instead on finding the optimal allocation within that band, combined with adaptive gating. If adaptive thresholds on aggressive geometry maintain hit-rate while improving latency, the composition strategy is vindicated and future cycles can explore entirely different optimization axes.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
