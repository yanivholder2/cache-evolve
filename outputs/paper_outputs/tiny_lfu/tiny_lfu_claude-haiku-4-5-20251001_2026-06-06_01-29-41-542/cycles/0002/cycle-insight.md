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
