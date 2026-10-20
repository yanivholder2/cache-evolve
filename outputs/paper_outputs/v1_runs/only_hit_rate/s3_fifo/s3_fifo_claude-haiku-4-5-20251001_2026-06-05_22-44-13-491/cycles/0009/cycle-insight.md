# Cycle 0009 Insight

## Cohort outcome

Three explorations probed the boundaries of the structure–frequency interaction space: incremental structural tightening to 1/15 paired with proven MAX_FREQ=5, extended frequency depth to MAX_FREQ=6 at extreme 1/14 selectivity, and a radically different trade-off pairing moderate 1/12 structure with aggressive MAX_FREQ=7. Both winners—the tighter structural variant and the deeper-frequency variant—yielded measurable gains, confirming the saturation zone remains productive in both dimensions. The loser decisively revealed that the optimization surface is asymmetric: aggressive frequency depth cannot recover performance lost by relaxing structural selectivity, indicating the peak lies in the extreme-structural region, not at moderate structure with extreme frequency.

## Patterns

- **Extreme structural selectivity remains the dominant axis even at saturation**: Both winners improved over the prior baseline, but v001's extension to 1/15 selectivity (paired with MAX_FREQ=5) and v002's frequency deepening to MAX_FREQ=6 (at proven 1/14) continued the pattern of nine consecutive cycles with zero degradation. This reaffirms that the workload rewards aggressive admission filtering throughout the measurable range. v003's failure—backing off to 1/12 with 2.33× deeper frequency protection—underscores that the optimization surface is not a smooth trade-off curve; items that never clear the tight admission gate cannot benefit from extended main-queue tenure, making structural selectivity the foundational axis and frequency depth a secondary multiplier dependent on structural aggressiveness.

- **Frequency-depth leverage plateaus at diminishing return magnitudes**: v002's +0.0003 gain from MAX_FREQ=6 represents the smallest improvement magnitude across the last three cycles, a near-halving of v001's +0.0007 or the prior cycle's +0.0014. This decay in frequency-depth returns suggests the main-queue retention mechanism is approaching saturation; items passing the extreme 1/14 gate are so carefully curated that further extending their tenure yields marginal benefit. The axis remains productive but at a steeply declining slope.

- **The optimization surface exhibits a discontinuous peak in the extreme-structural, moderate-frequency region**: v003's decisively larger loss (compared to the marginal wins of v001 and v002) demonstrates the surface is sharply peaked: the cost of relaxing structural selectivity from 1/14 to 1/12 is not recoverable by any frequency configuration tested to date. This mirrors Cycle 0007's inflection-point detection but with reverse asymmetry: then, frequency became "competitive"; now, frequency is revealed as subordinate to structure, unable to invert a poor structural choice. The peak appears to lie near or beyond the current 1/14–1/15 region with MAX_FREQ=5–6, not at broader structural ratios however deep their frequency protection.

## Open questions

- **Has structural selectivity finally hit its ceiling, or does 1/16–1/20 remain viable?** v001's +0.0007 gain at 1/15 is measurable but tiny; is the true saturation cliff beyond, or have we already crossed the peak?

- **What is the interaction-space peak in the extreme-structural zone?** The winners suggest different optimization directions (v001: deeper structure, v002: deeper frequency); is there a finer-grained optimum at 1/14.5-sized ratios (if continuous perturbation were possible) or combinations like 1/15+MAX_FREQ=6 not yet tested?

- **Why does the workload punish moderate structural selectivity so severely?** v003's loss with 1/12+MAX_FREQ=7 indicates that the reuse-depth clustering is so pronounced that items below the extreme admission threshold are predominantly noise; even with very deep frequency protection, they consume main-queue resources without value. Does this workload's access-pattern signature (histogram, skew, or temporal clustering metric) uniquely predict this regime?

## Continuation

Stop exploring moderate structural ratios; the asymmetry is now conclusive. Focus the next cycle on finely mapping the extreme-structural region (1/14–1/16) paired with moderate frequency depths (MAX_FREQ=5–7) to locate the true optimum. Test at least one unexplored pairing (e.g., 1/15+MAX_FREQ=6 or 1/14+MAX_FREQ=7) to check for interactions beyond the current grid, but commit to a dense sweep of 1/14–1/16×MAX_FREQ=5–7 if the next cycle yields saturating results. Once the peak is confidently mapped, pivot to characterizing the workload's access-pattern signature (reuse-depth histogram, inter-arrival clustering, frequency distribution) to test whether the extreme-selectivity requirement is predictable from raw access logs, or investigate learned/adaptive policies that dynamically adjust selectivity based on observed miss-pattern evolution in real-time rather than relying on fixed constants.
