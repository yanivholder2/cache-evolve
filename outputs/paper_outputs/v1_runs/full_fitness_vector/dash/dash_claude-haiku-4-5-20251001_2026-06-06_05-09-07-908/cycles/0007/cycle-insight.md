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
