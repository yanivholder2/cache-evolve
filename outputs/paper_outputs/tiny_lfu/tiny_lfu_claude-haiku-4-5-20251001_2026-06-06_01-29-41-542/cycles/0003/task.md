# Task: cycle-insight-writer

Cycle: 0003
Output dir: /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0003

## Winners

- v001

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0003/variants/v001/insight.md
- /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0003/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0003/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0003/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0003/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0003/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three variants tested different angles on the admission gate: sketch sizing, frequency decay, and gate threshold. Only the sketch-width increase (v001) improved hit rate; attempts to refresh frequency signal via decay (v002) or relax the gate threshold (v003) both degraded it.

## Patterns

- **Precision beats permissiveness**: The winner (wider sketch) improved frequency estimation fidelity while keeping admission logic unchanged. Both hurt variants tried to change the logic itself—either by decaying frequencies or accepting tied candidates—assuming the baseline was too strict. The data suggests the baseline's strictness is not the problem; measurement noise is.

- **Sketch width is a practical bottleneck**: Hash collisions in the Count-Min sketch directly degrade admission precision. Doubling the width factor (4→8) reduced collision errors and improved hit rate, confirming that sketch sizing, not responsiveness or recency effects, was limiting performance.

- **Early frequency signal is valuable**: v002's hypothesis that periodic decay would favor recent patterns over stale ones backfired—halving counters every capacity puts removes signal too aggressively. Items that earned high frequencies early are genuinely predictive, and monotonic accumulation preserves that signal better than periodic resets.

- **Ambiguity favors residents**: v003's relaxation from strict `>` to `>=` allowed tied frequency estimates to evict hot residents. When the sketch produces equal frequency estimates, accepting the tie admits marginal candidates based on recency alone, which harms hit rate. The strict gate's selectivity protects hot items during uncertainty.

## Open questions

- Does hit rate continue to improve with even wider sketches, or is there a diminishing return above 8x?
- Are other sketch parameters (depth, counter saturation strategy) worth tuning alongside width?
- Is the baseline's monotonic counter saturation at u32::MAX itself a limiting factor, or is width the only knob needed?

## Continuation

Pursue wider sketch sizing as the primary tuning axis—the baseline sketch was genuinely undersized. Drop hypothesis categories around decay, recency-weighting, and threshold relaxation; the admission logic is sound. For the next cycle, explore whether sketch depth matters, whether saturation saturation handling can improve further, or whether alternative frequency-estimation schemes (beyond Count-Min) might yield gains.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Three variants pursued sketch parameter tuning beyond Cycle 0001's width findings: testing width escalation to 16x, exploring depth as an orthogonal tuning axis, and combining both improvements. All three helped hit-rate, with v001's further width increase winning despite diminishing returns relative to prior cycles.

## Patterns

- **Width has entered diminishing returns**: The 4→8 increase from Cycle 0001 delivered +0.0047 gain; this cycle's 8→16 jump yielded only +0.0006. Doubling again produces an order of magnitude smaller payoff, signaling that pure width expansion is saturating.

- **Depth is a productive orthogonal axis**: v002's increase to 6 hash rows achieved modest but real improvement by strengthening independence of frequency estimates, orthogonal to width. This confirms that sketch parameter tuning has multiple directions, not just horizontal scaling.

- **Compound improvements plateau quickly**: v003's combination of both width and depth improvements (8x + 8 rows) delivered measurable gains, validating that the two dimensions interact minimally. However, the composite gain is modest—sketch parameter tuning as a whole is approaching its ceiling.

- **Frequency precision is no longer the bottleneck**: Cycle 0001's hypothesis that measurement noise limited admission accuracy has been mostly exhausted through width and depth tuning. The frequency-estimation surface has been substantially refined, yet hit-rate gains are flattening.

## Open questions

- Is there an optimal width-to-depth balance, or do they saturate independently?
- Are other sketch parameters (counter saturation strategy, hash seeding) worth tuning alongside width and depth?
- Should we pivot to alternative frequency-estimation structures (beyond Count-Min) to escape this plateau?
- Are other cache levers—admission gate logic, eviction ordering, or candidate selection—now the limiting factor?

## Continuation

Sketch parameter tuning has delivered measurable but declining gains. Width has plateaued; depth is productive but modest. Before pursuing further Count-Min refinements (saturation strategy, hash quality), consider whether a fundamentally different frequency-estimation approach would unlock larger gains. Alternatively, pivot to other cache levers: test whether the admission gate's logic (strict >) is still sound with refined frequency estimates, explore non-frequency signals (age, size, access pattern clustering), or revisit LRU ordering in light of improved frequency discrimination. The precise frequency surface has been tuned within Count-Min's limits; the next breakthrough likely lies outside sketch parameter space.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
