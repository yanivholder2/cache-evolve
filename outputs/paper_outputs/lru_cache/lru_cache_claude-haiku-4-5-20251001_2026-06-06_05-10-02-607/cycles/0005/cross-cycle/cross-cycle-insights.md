# Cross-Cycle Insights

## Recurring wins

- **Data structure efficiency beats algorithmic sophistication.** Cycles 0002–0005 consistently show that structural choices (bounded-window search, bucket indexing, pre-allocation) outperform fancy eviction strategies. The bounded-window foundation established in Cycle 0002 has held hit rate steady (0.6904) across all subsequent variants, suggesting the data structure itself, not eviction selection within it, is the primary determinant of performance.

- **Incremental work amortization outperforms operation-shifting deferral.** Cycle 0005's V002 (incremental queue draining) achieved 42% put p99 improvement while accepting 32% get p99 regression—a cleaner trade-off than Cycles 0003–0004's all-at-once deferrals, which produced 56% put regressions. Distributing work across multiple operations reduces peak spikes better than concentrating it into one.

- **Pre-allocation avoids mode-switching overhead.** Cycle 0005 showed that lazy dynamic allocation for mode-switching (V001) catastrophically failed (40× get p99 spike), while pre-allocated structures, though permanently present, avoid indirection and allocation costs. The fixed memory overhead is acceptable if it prevents pathological oscillation near threshold boundaries.

- **Workload-aware activation wins when conditions fire.** Cycle 0003's V003 (adaptive bucketing above saturation) achieved 34% get p99 improvement on the underutilized test case by staying in lightweight mode most of the time. The architecture is sound, but depends on validation that saturation thresholds actually trigger in the real domain.

## Recurring failures

- **Frequency tracking adds latency that scales with operation rate.** Cycles 0001, 0004 consistently show per-item frequency counters, multi-level ranking, and per-operation promotion checks improve hit rate marginally (if at all) but increase get and put latency universally. Cycle 0004's V003 gained only 1.3% hit rate while spiking put p99 2.8×. The metadata overhead is not amortized by improved hit rates on this workload.

- **Sampling and decision-logic overhead dominates on underutilized caches.** Cycles 0004–0005 show sampling counters, mode-transition checks, and saturation-tracking logic add measurable overhead (counter increments, modulo checks, conditional branches) that exceed any benefit on light-workload cases. Cycle 0004's V001 was "pure overhead" when decisions never triggered.

- **Deferral shifts cost between operations without eliminating it.** Cycles 0003–0005 repeatedly show that moving work from get to put (or vice versa) reliably improves one metric at the cost of another (23–56% put regressions in Cycle 0004; 32% get regression in Cycle 0005). Deferral is a trade-off tool, not a solution; whether it nets positive depends on the target's operation ratio and acceptable asymmetry.

- **Saturation-based thresholds become liabilities if they don't fire.** Cycles 0003 and 0005 added conditional activation logic (75% and 80% thresholds) that remained unused on test workloads that stayed underutilized (never exceeded 60%). Both paid decision-logic overhead without ever activating the feature. Pre-computed thresholds require validation in the real target domain.

- **Random sampling paradoxically underperforms deterministic bounded windows.** Cycle 0004's V003 used random sampling to avoid linear scans, but per-item frequency tracking and sampling variability made it unpredictable and slower than the deterministic bounded-window baseline. Determinism and structural simplicity trump theoretical worst-case reduction.

## Open questions

- **How to validate threshold-based designs before committing to their overhead?** Multiple cycles show conditional activation strategies (Cycles 0003, 0005) fail when saturation never triggers. Should future cycles stress-test these designs on longer traces or synthetic workloads with sustained high-saturation phases, or is the pattern signal enough to deprioritize threshold-dependent approaches?

- **Can frequency-aware eviction be made latency-efficient?** Frequency tracking consistently improves hit rate but adds latency overhead. Can bucketing, lazy updates, or adaptive decay reduce the cost without sacrificing hit rate, or is the overhead fundamental to selectivity?

- **Is V002's asymmetric trade-off (Cycle 0005: +42% put p99 vs. –32% get p99) acceptable for the target use case?** The answer depends on real production read-write ratios and which metric is the priority. Without deployment data, further optimization is speculative.

- **Do optimization opportunities exist only under sustained high saturation or adversarial access patterns?** The test workload is light and Zipfian, which may mask benefits of frequency tracking, adaptive modes, or sampling. Is the bounded-window baseline already sufficient for this domain, or would high-saturation phases unlock benefits that justify the overhead?

- **Can deferred work ever be amortized without shifting cost between operations?** All deferral strategies in Cycles 0003–0005 shifted cost from one operation to another. Can any buffering, batching, or prediction scheme improve both operations simultaneously, or is asymmetric cost-shifting inherent to deferral?

## Per-policy notes

- **Bounded-window sequential search (Cycle 0002 V001, baseline for 0003–0005).** Established as the reliable foundation: constant-time eviction discovery, predictable latency, stable hit rate (0.6904). Simple, structural approach outperforms all subsequent attempts at sophistication on the test workload. Prefers simplicity over selectivity.

- **Frequency-bucket indexing (Cycle 0002 V002).** Breakthrough winner: 15.6× put p99 improvement and gained hit rate (0.6904 vs. 0.6721). Proof that data structure reorganization can decouple hit rate from latency. Decaying overhead (bucket management) on the fast path compared to Cycles 0003+ variants; the original design was well-tuned.

- **Adaptive bucketing triggered by saturation (Cycle 0003 V003).** Most balanced approach: stays lightweight under underutilization (34% get p99 reduction vs. static bucketing), activates selectivity under pressure. Requires validation that saturation thresholds fire in real workloads; otherwise pays metadata and logic overhead without return.

- **Incremental queue draining (Cycle 0005 V002).** New practical winner: distributes deferred work across get and put, reducing peak spikes (42% put p99 improvement) while accepting measured latency cost. Outperforms all-at-once deferral from Cycle 0004. Architecture is sound; threshold tuning (queue depth, drain batch size) remains unexplored.

- **Per-operation frequency tracking (Cycle 0004 V003).** Universally penalizes latency despite marginal hit rate gains (1.3%). Counter increments and memory barriers dominate fast path. Confirmed that per-item overhead incompatible with latency predictability on this workload.

- **Random sampling (Cycle 0004 V003).** Paradoxically worse than deterministic bounded-window search. Theoretical worst-case improvement offset by sampling variability, per-item frequency overhead, and loss of structural predictability. Determinism wins on production workloads.

- **Lazy dynamic allocation for mode-switching (Cycle 0005 V001).** Catastrophic failure: allocation/deallocation cycles near saturation boundaries cause 40× get p99 spikes. Pre-allocation overhead is the price of avoiding mode-transition pathology. Memory optimization via lazy initialization backfires when boundaries oscillate.
