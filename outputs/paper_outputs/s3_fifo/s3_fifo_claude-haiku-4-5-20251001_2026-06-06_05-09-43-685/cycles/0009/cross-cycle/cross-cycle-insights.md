# Cross-Cycle Insights

## Recurring wins

- **Occupancy-responsive thresholds remain the most robust baseline**: Cycle 0003/v003 established that making promotion thresholds respond to queue occupancy delivers latency improvements without hit-rate loss. This design validated across Cycles 0005–0007 as a strong dual-metric baseline, even as other mechanisms failed.

- **Computational placement (hot vs. cold path) is the primary leverage point**: Expensive operations (occupancy computation, frequency decay, filtering logic) cause latency degradation when on hot paths (especially small-queue operations), but incur zero overhead when placed on cold paths (eviction decisions, ghost-hit re-admission). Validated consistently across Cycles 0008–0009; this is now the frontier.

- **Amortization unlocks richer state without cost**: Cycle 0009/v003 proved batch-amortizing expensive operations (frequency decay every N evictions) enables richer ghost-queue metadata and dramatic latency wins (44% faster gets, 37% faster puts) while maintaining hit-rate—key insight for future architectural choices.

- **Pre-computed ghost-queue metadata with eviction-time filtering**: Storing frequency depth or recency position in ghost entries, computed at eviction time (infrequent cold path), enables efficient downstream decisions. Both filtering (Cycle 0008/v001) and decay (Cycle 0009/v003) at eviction time are overhead-efficient.

- **Occupancy is a stable state signal**: Across multiple cycles attempting different signals (frequency distribution, age, recency), occupancy ratio consistently outperforms as a basis for adaptive decisions, requiring no workload-specific calibration.

## Recurring failures

- **Ghost-queue structural separation fails consistently**: Cycles 0003–0005 repeatedly showed that attempts to isolate frequency information into separate tracking structures (warm-ghost, deferred promotion) fail because the isolation breaks the coupling between frequency state and eviction decisions. Ghost works best integrated into the critical eviction path, not separated.

- **Per-operation occupancy-aware mechanics degrade latency**: Whenever occupancy-based logic is added to hot paths (Cycles 0004/v003, 0006/v001, 0006/v003, 0008/v003), latency suffers—occupancy ratios, conditional branching, and ratio-scaled decisions on every put/get compound into measurable overhead. Cold-path occupancy is safe; hot-path is not.

- **Fixed re-admission formulas don't adapt to workload state**: Cycles 0006–0007 showed fixed arithmetic boosts (e.g., `ghost_freq + 3`) overshooting some occupancy states and undershooting others, causing hit-rate loss during capacity transitions. Occupancy-responsive formulas (Cycle 0009/v002) adapt but require careful tuning.

- **Frequency distribution as a threshold signal is brittle**: Cycle 0004/v002 showed using frequency percentiles as promotion thresholds is less robust than occupancy—requires workload-specific calibration, adds sampling overhead, and is sensitive to access-pattern shifts.

- **Recency alone is a weak re-admission signal**: Cycle 0007/v003 achieved dramatic latency gains but at the cost of hit-rate loss—temporal distance from eviction does not capture re-access patterns as well as frequency or occupancy-based signals.

- **Relaxing ghost-queue admission filters reintroduces noise**: Cycle 0009/v001 tested relaxing aggressive filtering (freq > 0 → freq ≥ -1) and degraded put latency by 378ns p50 without recovering hit-rate, confirming Cycle 0008's tight boundary was correct.

## Open questions

- **Occupancy-responsive boost formula calibration**: Can Cycle 0009/v002's quadratic formula `(2.0 + occupancy_ratio²)` be tuned (lower magnitude, occupancy gating, frequency-percentile scaling) to recover hit-rate while preserving the 19% latency gain?

- **Combining orthogonal cold-path mechanisms**: Does Cycle 0009/v002's occupancy-responsive re-admission combined with v003's batch-amortized decay unlock dual-metric wins? The two succeed independently; their interaction is untested.

- **Amortization window parameter space**: Is Cycle 0009/v003's 32-eviction batch window optimal for this workload, or do different batch sizes (16, 64) balance decay freshness against amortization overhead differently?

- **Multi-signal re-admission with adaptive weighting**: Can frequency + occupancy (or frequency + recency) signals with workload-responsive weights improve re-admission prediction beyond single-signal approaches?

- **True Pareto frontier boundaries**: Is the current latency–hit-rate tradeoff fundamental, or do unexplored mechanisms (recency weighting in ghost, hybrid frequency-occupancy scoring, per-zone adaptive thresholds) exist that improve both metrics simultaneously?

- **Ghost-frequency decay retention depth**: Across decay strategies tested (depths from -1 to -3), what is the optimal retention threshold that preserves warm candidates without per-operation cost?

## Per-policy notes

- **Three-tier architecture (main + small + ghost) supports occupancy-responsive design**: All winning variants leverage this structure. Main-queue occupancy ratio drives threshold adaptation; ghost queue stores pre-computed frequency or position metadata; small queue absorbs low-frequency working set. The tiers are coupled via promotion decisions—occupancy is the signal that coordinates them.

- **Small-queue is the latency-critical path**: Per-operation overhead on small-queue puts and gets degrades both operations most severely. This is the hot path; any mechanism added here must be overhead-free. Cycles 0004–0009 consistently show this bottleneck.

- **Eviction-time decisions are architectural zero-cost**: Filtering (Cycle 0008/v001: freq > 0 ghost filter) and decay (Cycle 0009/v003: batch-amortized frequency decay) at eviction time incur no measurable latency penalty because evictions are infrequent relative to hits. This is the safe place to add complex logic.

- **Ghost-queue re-admission is a cold-path signal opportunity**: Ghost hits are rarer than hits to live entries; occupancy-responsive re-admission formulas (Cycle 0009/v002) can be expensive here without hot-path cost, unlike main-queue promotion decisions (which must be overhead-free).

- **Occupancy stability enables parameter-free adaptation**: Unlike frequency distribution (workload-sensitive, requires calibration) or recency (poor predictor), occupancy ratio is a workload-agnostic measure of queue load. Multiple cycles confirm it requires minimal tuning across different workloads.
