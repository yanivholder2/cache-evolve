# Cross-Cycle Insights: LRU Cache Eviction Optimization

## Recurring wins

- **Data structure choice dominates over eviction strategy choice**: Bounded search windows (Cycles 0002, 0006) and frequency-level bucket indexing (Cycles 0002, 0007) consistently outperform algorithmic strategies that rely on expensive linear scans. When the eviction mechanism is architecturally sound, latency improves predictably; when it is not (e.g., per-item frequency counters requiring search, V003 in Cycle 0006), even clever selectivity fails.

- **Deferred-work amortization across multiple operations beats concentrated flushing or single-operation intrusion**: Cycle 0007's two-batch swap mechanism achieved simultaneous improvement in both get and put p99 (42% and 50%, respectively) by coalescing deferred work into batched flushes. This breaks the operation-shifting pattern that plagued Cycles 0004–0006, where deferral always moved cost from one operation to another.

- **Hit rate plateaus once the eviction mechanism is sound**: Across all eight cycles, hit rate is stable (~0.66–0.69) regardless of whether frequency tracking, recency counting, or window sizing is optimized. Window size and eviction strategy fundamentally determine selectivity; beyond that foundation, further sophistication in candidate selection does not improve hit rate on this workload.

- **Workload-aware conditional activation (saturation, mode switching) can reduce overhead when thresholds actually fire**: Cycle 0003's adaptive bucketing achieved 34% get p99 improvement by activating expensive logic only above 75% saturation. However, success requires accurate threshold prediction and validation that conditions fire in the real workload; blind threshold assumptions cause failures (Cycle 0007 V003: 9.4% hit-rate drop when deferred promotions broke LRU at low saturation).

## Recurring failures

- **Sampling and conditional logic in light-load workloads introduce pure overhead**: Cycle 0004's periodic sampling (V001) and random sampling with frequency tracking (V003) both regressed sharply because the underutilized workload made infrequent eviction decisions and rarely triggered high saturation. Optimization infrastructure designed to amortize cost incurs checking/bookkeeping overhead that never pays off.

- **Dynamic allocation for mode-switching near saturation boundaries causes catastrophic latency spikes**: Cycle 0005's lazy bucket allocation (V001) suffered 40× get p99 regression when the workload oscillated near saturation boundaries, triggering repeated allocation/deallocation cycles. Pre-allocated structures, while permanently present, avoid the pointer indirection and allocation overhead that lazy initialization paid on every mode transition.

- **Hot-path intrusion for work amortization negates benefit faster than deferral alone**: Cycle 0006's incremental queue draining during get() added drain-trigger checks to the critical path, causing 34% get p99 regression despite reducing put p99 by 42%. Cycle 0008's per-operation saturation check achieved 36% get p99 improvement but at cost of 31% get p50 regression. Condition checking in hot paths must be nearly free or avoided entirely.

- **Correctness-critical operations (LRU promotion, eviction ordering) cannot be deferred without universal activation or semantic preservation**: Cycle 0007's saturation-aware deferred promotion (V003) attempted to defer LRU promotions at low saturation, breaking LRU ordering and causing a 9.4% hit-rate drop (0.6904→0.6255). Conditional deferral of invariant-critical operations requires either universal activation or mode-conditional semantics that preserve baseline behavior when inactive.

- **Optimizations are not orthogonal across contexts; removal of supporting features breaks amortization justification**: Cycle 0008's V002 attempted to decouple batched queue-flush from frequency tracking by switching to pure FIFO. The result was catastrophic: get p99 regressed 203%, hit rate dropped 4.8%, and metadata actually increased. Batching amortizes cost because frequency-aware decisions justify the batch-processing overhead; without that selectivity benefit, batching is pure overhead.

## Open questions

- **What is the actual saturation profile of the test workload?** Optimization strategies across Cycles 0003–0008 relied on saturation assumptions (75% threshold, 45% threshold, 80% threshold) that did not fire or rarely fired. Validating the workload's saturation distribution would clarify whether deferred-work and mode-switching optimizations are worth the infrastructure cost.

- **Can hit rate be improved beyond ~0.69 without per-item frequency tracking?** V002 in Cycle 0008 showed 4.8% hit-rate degradation when frequency tracking was removed, yet Cycle 0007's V002 (pure FIFO with larger window) maintained hit rate unchanged. Does window size fully capture selectivity, or does frequency tracking provide non-eliminable benefit on non-Zipfian workloads or under sustained saturation?

- **Can hot-path condition checking (saturation thresholds, mode selection) be computed asynchronously or sampled to avoid per-operation overhead?** Cycles 0006 and 0008 showed that threshold checks introduce measurable overhead on median latency. Does periodic sampling, counter-based approximation, or asynchronous tracking of saturation reduce this cost while preserving the tail-latency benefit?

- **Can V001's batched queue-flush mechanism from Cycle 0007 be applied to frequency-free eviction strategies?** Cycle 0007's two-batch swap achieved dual p99 improvements with deferred promotions; Cycle 0008 showed frequency tracking is eliminable. Can batching coexist with pure FIFO to achieve both latency gains and hit-rate preservation?

- **Is the 20% get p50 regression from Cycle 0007's V001 tunable through batch-size, swap-frequency, or drain-threshold optimization?** The queue-swap mechanism's overhead is measurable but bounded. Aggressive parameter tuning might recover p50 performance while preserving p99 gains.

## Per-policy notes

- **Frequency-recency hybrid eviction (Cycles 0001–0002, 0006)**: Strong hit-rate and get-latency gains when paired with efficient data structures (buckets, bounded windows). However, per-operation frequency tracking introduces overhead that becomes a bottleneck when hot-path condition checking or per-item increments are required. Success depends on amortizing frequency updates; when updates are deferred algorithmically (batched flush), this strategy wins; when updates are checked on every operation, it fails.

- **Bounded-window sequential search (Cycles 0002–0008)**: Latency-predictable baseline that avoids pathological O(N) scans. Pareto-optimal for light workloads when window size captures the working set. Success requires window size calibration; too-small windows miss hit-rate gains, too-large windows increase eviction cost.

- **Adaptive saturation-aware mode switching (Cycles 0003–0008)**: Sound in principle—minimize overhead at light load by activating expensive logic only when saturation is high. Fragile in practice: requires accurate saturation threshold prediction, correctness-preserving inactive behavior, and validation that mode-switching conditions actually fire in the real workload. Multiple failures when workload saturation profile differed from assumptions.

- **Deferred updates and promotions (Cycles 0003–0008)**: Can improve p99 tail latency when amortized algorithmically (batched flush, two-batch swap in Cycle 0007), but introduce operation-shifting trade-offs (Cycles 0004–0006). Benefit is largest when deferred work is frequent and amortization mechanism is efficient; cost is highest when deferral conditions are unmet or when the deferral itself corrupts invariants (Cycle 0007 V003).

- **FIFO with larger fixed window (Cycle 0007 V002)**: Validates that hit rate is window-size-dependent and frequency tracking is eliminable. Marginal latency improvements suggest frequency-tracking overhead is not a bottleneck on underutilized workloads; simpler algorithms do not significantly improve latency even when overhead is removed.

- **Metadata overhead (frequency counters, recency tracking, saturation flags, batch vectors)**: Consistently ranges 8–72 bytes per item or cache struct. Never a direct latency bottleneck when cold (unused), but hot-path operations (counter increments, searches, threshold checks, vector allocations) that depend on metadata introduce costs that often exceed removal gains unless the workload is CPU-bound or memory-starved.
