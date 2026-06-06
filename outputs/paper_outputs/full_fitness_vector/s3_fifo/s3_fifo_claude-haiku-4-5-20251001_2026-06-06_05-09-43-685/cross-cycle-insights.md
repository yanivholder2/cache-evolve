# Cross-Cycle Insights

## Recurring wins

- **Occupancy-responsive threshold adaptation**: Cycles 0003, 0007–0009 consistently demonstrated that promotion thresholds responding to main-queue occupancy unlock latency improvements without sacrificing hit-rate. This contrasts sharply with fixed tight thresholds (cycles 0001–0002), which trade hit-rate for latency. The occupancy signal is more durable than frequency-distribution percentiles and enables state-responsive decisions at zero hot-path overhead.

- **Cold-path computational placement as leverage**: Cycles 0009–0010 validated that moving expensive operations (occupancy computation, frequency decay) off hot paths enables richer ghost-queue state without per-operation cost. Batch amortization proved particularly effective when aligned with small-queue eviction frequency, delivering dramatic latency gains while preserving hit-rate.

- **Ghost-queue frequency filtering at eviction time**: Cycles 0008–0009 established that filtering ghost entries to preserve only frequently-accessed items (freq > 0) is a robust latency lever with minimal implementation overhead. This boundary point was validated through relaxation experiments showing that looser retention (freq ≥ -1) reintroduced search overhead without gain.

- **Pre-computed ghost metadata for efficient re-admission**: Cycles 0006+ consistently showed that storing state tuples in ghost entries enables cold-path re-admission decisions without expensive runtime lookups, provided operations are computationally cheap (arithmetic, bit flags, not floating-point or complex branching).

## Recurring failures

- **Per-operation structural adaptation is fundamentally expensive**: Cycles 0004–0005 demonstrated that dynamic queue sizing with per-operation overhead (occupancy tracking, resizing logic) degrades latency despite hit-rate benefits. The cost scales with operation frequency and is unavoidable through parameter tuning—avoiding this path entirely is more productive than attempting to optimize it.

- **Frequency-decay tracking on lean baselines**: Cycles 0005, 0007 showed that attempts to preserve frequency information via decay tracking without structural backing either corrupt admission signals (loose retention semantics) or reintroduce per-operation overhead that erases intended gains. Ghost-frequency preservation requires either batch amortization (cycle 0009/v003) or deeper architectural redesign; intermediate approaches fail.

- **Frequency-distribution as a signal**: Cycle 0006 proved that frequency distribution percentiles (75th percentile threshold) require workload-specific calibration and are sensitive to access-pattern shifts. Occupancy—which directly reflects queue state—proved more robust and does not demand percentile recomputation.

- **Parameter tuning to escape local optima**: Cycles 0009–0010 revealed that fine-grained threshold tuning (0.4/0.7 vs 0.5/0.8), linear vs quadratic boost scaling, and other adjustments to existing mechanisms explore noise rather than new optima. Five consecutive cycles of parameter refinement yielded only trade-offs, suggesting the search space around established mechanisms is exhausted.

- **Implementation complexity on hot paths**: Cycle 0010 demonstrated that architecturally sound mechanisms (frequency-threshold gating for re-admission selectivity) collapse under implementation cost if they require expensive operations (floating-point arithmetic, extended data structures, complex conditionals) on hot paths. Cheap operations scale; complex ones do not.

## Open questions

- **Can frequency-selective re-admission be optimized for latency?** Cycle 0010/v003 proved that selective re-admission (only re-admitting items with freq > threshold) correctly prevents low-frequency ghost items from crowding the working set, yielding a +1.14% hit-rate gain. However, implementation overhead (extended ghost tuples, frequency lookups) caused 36% get latency degradation. Can inline frequency storage, bit-packing, or sampled checks recover latency while preserving the hit-rate improvement?

- **Has a true Pareto frontier been reached?** Five cycles of signal innovation, placement optimization, and parameter refinement have yielded only trade-offs. Does an alternative mechanism (recency weighting, bimodal admission, frequency-decay scheduling independent of eviction) exist that could improve both latency and hit-rate, or is the frontier fundamental to the queue structure?

- **What metric priority should drive next exploration?** Cycle 0009/v003's baseline delivers 44%+ latency improvements with negligible hit-rate cost (<0.1% degradation). Should future cycles pursue latency specialization (production-ready) or invest engineering effort in hit-rate recovery mechanisms?

- **Can occupancy-responsive mechanisms be made implementation-efficient?** Occupancy signals proved architecturally sound and enable adaptive re-admission, but floating-point calculations carry hidden overhead (cycle 0010/v002). Are there low-cost proxy signals (occupancy buckets, bit flags, coarse thresholds) that preserve adaptive benefits without computation cost?

- **Can batch-amortized decay parameters be tuned?** Cycle 0009/v003's 32-eviction window proved effective, but is it optimal? Should decay depth (−2 vs −3) or amortization frequency be varied to maximize hit-rate recovery or reduce decay staleness?

## Per-policy notes

- **Promotion-threshold policy**: Fixed tight frequency thresholds (cycles 0001–0002) achieve latency but sacrifice hit-rate significantly. Occupancy-responsive thresholds (cycles 0003, 0007–0009) decouple metrics and represent the strongest baseline for balanced performance. Further parameter refinement of fixed or occupancy-responsive thresholds shows diminishing returns; mechanism innovation is required for new progress.

- **Ghost-queue filtering policy**: Eviction-time filtering (cycles 0008–0009) is overhead-efficient and should be the foundation for ghost management. Cold-path re-admission decisions enable adaptive signals (occupancy-responsive boosts, frequency-selective gating) without hot-path cost, provided operations remain computationally cheap. Preservation of frequency depth through batch amortization (cycle 0009/v003) avoids the signal-corruption problem that loose retention creates.

- **Queue structure and sizing**: Fixed proportions (main/small queue ratios) with occupancy-responsive thresholds outperform dynamic resizing approaches (cycles 0004–0005, 0010). Per-operation structural adaptation is a losing direction; the architecture stabilized in cycles 0003–0006 and should not be revisited unless the workload fundamentally shifts.

- **Frequency tracking and decay**: Per-operation decay tracking fails (cycles 0005, 0007); batch amortization succeeds (cycle 0009/v003). This distinction—whether state tracking happens per-operation or per-batch—is more consequential than the specific decay formula or retention depth. When frequency preservation is required, batch amortization is the only viable approach at acceptable latency cost.
