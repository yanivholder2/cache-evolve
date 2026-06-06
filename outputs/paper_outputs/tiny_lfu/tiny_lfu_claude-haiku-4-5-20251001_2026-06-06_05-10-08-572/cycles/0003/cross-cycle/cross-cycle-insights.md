# Cross-Cycle Insights

## Recurring wins

- **Sketch geometry stabilization through systematic exploration**: Cycles 0001–0003 converged on a stable depth-3, width-4 baseline by testing orthogonal dimensions (width then depth then width-on-depth). Each geometry change that advanced did so by identifying cost-benefit sweet spots rather than unbounded expansion.
- **Frequency estimation quality as the primary lever**: Every winner in the cohort improved hit-rate or tail latency, both metrics downstream of frequency accuracy. Cycles 0001–0003 confirmed that collision reduction (width) and temporal freshness (decay) both pay off, while geometries that don't improve frequency estimates (pure depth expansion in 0002, wide-width-on-depth in 0003) degrade.
- **Tail latency and hit-rate as superior fitness signals**: Median latency is misleading; only variants that improved p99/put operations or hit-rate alongside per-op latency proved durable. Cycle 0002's p50-favorable but p99-unfavorable depth reduction and cycle 0003's v001 (hit-rate up, latency way up) both flagged misdirected tuning.

## Recurring failures

- **Admission threshold micro-tuning is brittle and non-monotonic**: Cycles 0002 and 0003 both attempted to relax or offset the strict `>` threshold (v003 in 0002 with `>=`, v002 in 0003 with `>= victim - 1`). Both failed identically: marginal relaxations admitted lower-temperature items that displaced higher-value residents, hurting hit-rate. The threshold encodes workload-specific temperature distribution and has no smooth gradient; attempts to find "better" conditions via small offsets are futile.
- **Depth expansion above 4 shows diminishing returns**: Cycle 0002's v003 (depth 4→6) yielded only +0.33% hit-rate for 11% latency cost and 50% memory overhead. This contradicts the pattern that collision reduction is high-ROI; indicates width-based collision density, not depth, is the bottleneck at this workload scale.
- **Width expansion on shallow baselines hits cost barriers**: Cycle 0003's v001 (width 4→8 on depth-3) improved hit-rate by ~0.6% but surged latency 60% and doubled memory, negating the latency savings from depth-3. Width expansion in cycle 0001 was high-ROI; cycle 0003 suggests the cost-benefit ratio is cache-capacity and geometry dependent, with width-4 near-optimal for this setup.

## Open questions

- **Does sketch decay synergize with stable geometry (depth-3, width-4)?** Cycle 0003's v003 (decay) improved both hit-rate and latency simultaneously, a breakthrough. Testing decay on the proven stable baseline could compound gains; currently only tested against v001's oversized width.
- **What is the optimal decay frequency, and does it generalize?** Cycle 0003 used ~500k operation intervals. Bracketing this range (250k, 1M) would identify workload sensitivity; scaling to different cache sizes and Zipfian skews is unresolved.
- **Is the admission threshold's brittleness a true constraint or a symptom of staleness?** Cycles 0002–0003 failed to relax it, but cycle 0003's decay suggests frequency estimate freshness underlies responsiveness. If decay removes phase-transition noise, does the strict threshold become less essential or more obvious as a workload-specific constant?
- **How do findings scale to denser workloads or different Zipfian skews?** All three cycles tuned on one trace. Frequency-based decisions proved dominant on this trace; lower skew or bursty patterns may shift bottleneck to operational latency or admission selectivity.

## Per-policy notes

- **Frequency-based admission is sensitive to estimate staleness and workload temperature distribution**: The policy's performance hinges on two orthogonal factors—collision-free hashing (geometry) and temporal freshness (decay). Geometry alone saturates; decay opens new headroom.
- **Admission selectivity (threshold) is a local optimum with no smooth relaxation path**: The strict `>` condition appears tailored to workload phase structure. Gradual relaxation does not interpolate between strict and permissive; it causes cliff-edge hit-rate degradation. This suggests future tuning should either fix the threshold for a given workload class or investigate workload classification to adapt it.
- **Memory-latency tradeoffs in sketch geometry are non-linear and cache-scale dependent**: Width expansion is high-ROI up to a point (width-4 appears stable for this cache size), then hits cost barriers. Depth reduction yields latency gains with minimal hit-rate loss (depth-3 sweet spot), but expansion is wasteful. The tradeoff curve is not smooth.
