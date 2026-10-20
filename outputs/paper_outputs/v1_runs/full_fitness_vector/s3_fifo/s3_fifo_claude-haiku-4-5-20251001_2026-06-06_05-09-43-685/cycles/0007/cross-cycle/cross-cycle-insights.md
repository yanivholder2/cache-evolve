# Cross-Cycle Insights

## Recurring wins

- **Occupancy-responsive threshold adaptation** (cycles 0003, 0007): Making promotion-frequency thresholds responsive to main-queue occupancy unlocks latency gains without sacrificing hit-rate, decoupling metrics that fixed thresholds couple. This pattern held across baseline (cycle 0003/v003) and was extended successfully into adaptive re-admission signaling (cycle 0007/v002).

- **Tight admission filtering as latency lever** (cycles 0001–0005): Stricter promotion thresholds consistently improve operation latency by reducing main-queue contention and lookup overhead. The mechanism is reliable but carries a fundamental hit-rate cost that requires compensation through other channels.

- **Pre-computed ghost-queue state with adaptive signals** (cycles 0006–0007): Storing frequency/metadata in ghost entries and computing re-admission signals on retrieval enables both latency and hit-rate improvements, outperforming fixed-boost or purely structural approaches. Success depends critically on the signal (occupancy-responsive scaling works; fixed arithmetic does not).

- **Occupancy as the primary queue-state signal** (cycles 0003–0007): Occupancy (fractional main-queue fullness) proved more robust and lower-overhead than frequency distribution or age-based metrics for informing admission and re-admission decisions. Direct observations of queue state outscale workload-dependent heuristics.

## Recurring failures

- **Ghost-queue frequency preservation without decay discipline** (cycles 0005, multiple variants): Attempts to relax ghost-queue retention to recover hit-rate (loosening threshold to freq ≥ -1, full frequency preservation) corrupted admission signals because inflated frequency scores contradicted actual re-access likelihood. The coupling between decay rate, retention threshold, and re-admission weight is tight; tuning one lever alone destroys the signal.

- **Per-operation overhead in structural adaptation** (cycles 0004–0006): Occupancy-aware queue resizing, age-aware thresholds, frequency-distribution computation, and per-operation adjustments all add latency (36–244ns penalties) that eroded hit-rate gains. Threshold adaptation (cycle 0003) was overhead-free; any downstream per-operation logic compounds cost. Tuning parameters (e.g., finer occupancy breakpoints) adds cost without marginal benefit once a design is locally optimal.

- **Recency as sole re-admission signal** (cycle 0007/v003): Entry position in ghost is computationally cheap (43–57% latency improvement) but predictively weak—temporal distance from eviction does not capture actual frequency distribution or re-access patterns, resulting in hit-rate loss. Recency lacks the semantic connection to workload access that frequency or occupancy provides.

- **Ghost-frequency decay on lean baselines** (cycle 0007/v001): Layering ghost-frequency preservation (-3 depth decay) onto a lean architecture reintroduced per-operation search and boosting overhead without recovering hit-rate. Passive frequency preservation cannot substitute for active barrier adaptation; the mechanism is sound but contexts matter.

- **Incremental parameter tuning past local optima** (cycle 0006/v001): Fine-grained occupancy-breakpoint tuning (0.4, 0.7 vs. 0.5, 0.8) yielded negligible gains once cycle 0003/v003's parameters were well-calibrated. Search space was overfitted; further refinement explores noise, not new optima.

## Open questions

- **Can v002's adaptive re-admission boost (cycle 0007) be further tuned or combined with additional signals?** Occupancy-responsive scaling of the re-admission boost works; can the scaling coefficients, formula (e.g., `boost = 2 + occupancy_ratio`, `boost = 2 + frequency_percentile`), or decay depth be optimized further, or would signal combination (frequency + recency, occupancy + recency) recover both latency and hit-rate?

- **Is there a hybrid re-admission mechanism that combines frequency and recency?** v003 (cycle 0007) showed recency is efficient but weak; could prioritizing recently-evicted items within frequency-aware ghost retention recover latency while preserving hit-rate?

- **Has the cohort reached a true Pareto boundary, or does an unexplored mechanism exist?** Occupancy-responsive thresholds and adaptive ghost re-admission have delivered consistent gains; could recency-weighted frequency scoring, bimodal frequency thresholds, or decay-rate adaptation unlock further improvement without incurring per-operation overhead?

- **What is the optimal ghost-queue frequency decay depth?** Cycles explored -2 (v002, cycle 0006–0007) and -3 (v001, cycle 0007); is there a sweeter point (e.g., -1 for aggressive filtering, -2.5 for balance) that reduces per-operation overhead while preserving warm candidates?

- **Can per-operation overhead costs be amortized rather than eliminated?** Sampling-based occupancy tracking, lazy evaluation, or batch resizing might reduce per-operation cost without sacrificing decision quality; do such approaches exist and would they enable hit-rate recovery through structural sizing?

## Per-policy notes

- **Promotion-threshold adaptation**: Occupancy-responsive threshold scaling (freq > 3 when occupancy > 80%, freq > 1 when occupancy < 50%) is the most efficient lever—zero per-operation overhead, dual-metric wins, scales well. This policy has held as the strongest baseline across cycles 0003–0007.

- **Ghost-queue re-admission signaling**: Fixed arithmetic boosting (e.g., `ghost_freq + 3`) works only when aligned with specific decay depth; occupancy-responsive scaling (e.g., `boost = 2 + occupancy_ratio`) adapts to workload transitions and avoids signal overshoot. Pre-computed ghost state is essential for latency efficiency; the re-admission formula is the binding variable.

- **Ghost-queue retention and decay**: Frequency preservation requires disciplined decay (e.g., -2 depth with strict re-admission weight) to avoid signal corruption. Naive retention (freq ≥ -1, full preservation) fails; aggressive filtering (-3+) improves latency but sacrifices hit-rate recovery. Balancing retention depth with re-admission formula is critical.

- **Queue structural changes**: Resizing small-queue capacity, adjusting eviction timing, or frequency-distribution-based thresholds consistently introduce per-operation overhead that undermines hit-rate gains. These mechanisms are less efficient than threshold adaptation; further investment should be conditional on amortization strategies.

- **Frequency distribution as signal**: Frequency percentiles (e.g., 75th percentile threshold) require workload-specific calibration and are sensitive to access-pattern shifts. Occupancy (direct queue state) is more robust and lower-cost; frequency distribution is a less stable choice for adaptive policies.
