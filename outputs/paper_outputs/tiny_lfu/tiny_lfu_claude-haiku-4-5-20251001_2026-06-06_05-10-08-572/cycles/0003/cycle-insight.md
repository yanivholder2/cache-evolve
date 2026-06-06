# Cycle 0003 Insight

## Cohort outcome

This cycle tested three orthogonal hypotheses: width-4-to-8 expansion on depth-3 (v001), micro-tuning the admission threshold with a -1 offset (v002), and periodic frequency sketch decay (v003). **Only sketch decay advanced as a winner**—the first variant in three cycles to improve both hit-rate and per-operation latency simultaneously. Width expansion proved counterproductive, and admission threshold tuning remains brittle. The cohort reveals that sketch geometry has reached saturation while temporal freshness of frequency estimates opens a new, unexplored dimension.

## Patterns

- **Width expansion hits diminishing returns and cost barriers on depth-3 baseline**: v001 doubled width (4→8) to recover hit-rate lost in cycle 0002's depth reduction, successfully raising hit-rate by ~0.6%. However, per-operation latency surged 60% (90→144 ns) and memory doubled, negating the latency gain that depth-3 achieved. This contradicts cycle 0001's finding that width expansion is high-ROI, suggesting the cost-benefit ratio is cache-size and workload specific. At this cache capacity, width 4 is near-optimal.

- **Admission threshold remains a local optimum with no smooth gradient**: v002 tested a -1 offset (>= victim_freq - 1) as a middle ground between strict (>) and permissive (>=). It failed predictably: the relaxation admitted marginal items that displaced higher-value cache residents, hurting hit-rate (opposite of the hypothesis). This mirrors cycle 0002's failure with >=, indicating the strict inequality encodes workload-specific temperature distribution knowledge that marginal relaxations cannot overcome. The threshold is brittle and workload-dependent.

- **Sketch decay is a breakthrough orthogonal axis**: v003's periodic decay (every ~500k operations) improved both hit-rate and latency simultaneously, a first across three cycles. Hit-rate gains are modest but consistent (responsive to workload phase changes), while latency improvements are substantial—both get (132→112 ns) and put (166→142 ns) operations accelerated. The mechanism appears to be two-fold: (1) temporal freshness for adaptive admission, and (2) reduced computational overhead as unbounded count growth is periodically reset. This suggests decay addresses a bottleneck orthogonal to sketch geometry.

- **Latency-accuracy tradeoffs are not absolute**: Most prior optimizations traded one metric for another. v003 broke this pattern by improving both, hinting that sketch staleness is a distinct constraint from collision-density (geometry) or selectivity (admission).

## Open questions

- Does sketch decay synergize with width-4, depth-3 geometry (the likely stable baseline)? v003 was tested against the baseline; combining it with known-good geometry might compound gains.
- What is the optimal decay frequency for this workload? v003 used ~500k operations; is this interval universal or workload-dependent? Longer intervals might reduce overhead further; shorter intervals might improve phase responsiveness.
- Does decay scale to different cache sizes and Zipfian skews? All variants have been tuned on one trace; decay's responsiveness benefit may be more valuable on bursty or phase-shifting workloads.
- Why did admission micro-tuning fail when sketch geometry tuning succeeded? Is the threshold truly load-bearing, or are the workload's phase transitions (which decay now handles) the hidden reason prior threshold tuning failed?

## Continuation

**Depth-3, width-4 baseline is near-optimal for sketch geometry**; v001's width expansion proved too costly, and v002's depth reduction already extracted the latency gain. Stop pursuing geometry changes. **Decay has opened a new frontier**: v003's co-benefits (hit-rate + latency improvement, not just tradeoffs) suggest temporal freshness is a first-class tuning lever. For cycle 0004, test decay decay-frequency tuning (e.g., 250k and 1M operations to bracket v003's 500k), and consider pairing optimal decay with a stable geometry baseline to measure compounding effects. **Abandon admission threshold micro-tuning**—the strict `>` condition is workload-optimized and has no smooth gradient. Shift focus entirely to decay mechanics and interactions with sketch geometry.
