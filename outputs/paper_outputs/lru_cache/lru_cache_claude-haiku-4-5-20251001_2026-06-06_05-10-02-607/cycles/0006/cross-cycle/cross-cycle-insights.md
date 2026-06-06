# Cross-Cycle Insights

## Recurring wins

- **Bounded-window eviction search is a robust latency foundation.** Cycles 0002–0006 consistently show that pre-structured search windows (fixed scan depth or bucketed metadata) eliminate O(N) pathological spikes and provide predictable tail latency. This structural choice dominates the particular eviction strategy within the window.

- **Defer work from hot path to cold path.** Complete deferral of deferred work (e.g., promotion updates) from get() to put() beats incremental amortization (Cycle 0006: V002 gains 59% get p99 by eliminating drain checks, accepting modest put cost). The asymmetry in operation frequency (gets >> puts) makes this consistently valuable.

- **Hot-path minimalism outperforms architectural sophistication.** Cycles 0005–0006 show that removing checking/branching logic from the most-executed operation wins more than clever conditional activation or batching mechanisms. A clean hot path with occasional bulk work on the cold path beats intricate amortization.

- **Hit rate plateaus early once bounded window is established.** Cycles 0002–0006 maintain stable hit rates (0.6904, 0.6652, 0.6987) across diverse eviction-selection strategies. The bounded-window baseline is already near-optimal for this workload's Zipfian skew; further selectivity sophistication does not materially improve hit rate.

## Recurring failures

- **Per-operation frequency/recency tracking regresses tail latency predictably.** Cycles 0003–0004, 0006 show that adding counter or tracking fields to nodes, regardless of how clever the eviction logic, creates latency overhead on the critical path that outweighs selectivity gains. V003 in Cycle 0004 (per-item frequency) caused 2.8× put p99 spike; V003 in Cycle 0006 (operation-count recency) introduced a latency cliff via windowed min-search.

- **Incremental work on the hot path hurts more than bulk work on the cold path.** Cycles 0005–0006 repeatedly show this: drain-trigger checks on get() add checking overhead that exceeds the put-side benefit, even when put operations are significantly optimized. V002 in Cycle 0006 proved bulk flushing at put (all work batched) beats V001's incremental draining on get (34% regression despite 18% put p50 gain).

- **Conditional/saturation-based mode switching costs more than benefit on underutilized workloads.** Cycles 0003 and 0005 show V003 variants adding metadata overhead and decision logic without ever triggering the expensive mode (saturation thresholds at 75–80%, workload stays <60%). The architecture is sound but requires workload validation; mispredicted thresholds become pure overhead.

- **Lazy allocation for mode-switching introduces catastrophic oscillation costs.** Cycle 0005 V001's dynamic `Option<Box<>>` allocation caused 40× get p99 spike when workload oscillated near saturation boundaries, creating repeated allocation/deallocation cycles. Pre-allocated structures, while permanently present, avoid the pointer indirection and allocation overhead.

- **Eviction-candidate search within windows is a latency bottleneck regardless of search signal.** Cycle 0006 V003 attempted to replace frequency counters with operation-count recency, requiring a min-search over the bounded window during eviction. The search operation itself (cache misses, pointer chasing) regressed latency more than the counters it replaced. Bounded-window sequential scan is cheaper than explicit min/max operations.

## Open questions

- **Does hit rate actually plateau on this workload, or is the domain just too simple/underutilized to reveal selectivity gains?** Multiple cycles show hit rate unchanging despite diverse eviction strategies. Is the Zipfian skew strong enough that simple LRU within a bounded window already captures the hot set, or would a write-heavy, non-Zipfian, or sustained-saturation workload show that frequency tracking does matter?

- **Can deferred-work trade-offs be tuned closer to Pareto optimality?** Cycle 0006 V002 gained 59% get p99 but regressed put by ~18 ns. Can queue drain thresholds, batch sizes, or burst mechanisms be tuned to improve both simultaneously, or is the get/put trade-off fundamental to deferral?

- **What concrete conditions trigger the latency cliff in Cycle 0006 V003's operation-count search?** Was it cache misses during the windowed scan, or pointer-chasing overhead on `last_access_count` fields, or eviction-candidate density spikes? Would approximate or sampled recency (e.g., binned access epochs) avoid the search cost while retaining selectivity?

- **How much put-latency regression is acceptable in production for a given read/write ratio?** All recent cycles show get improvements at put's expense. The trade-off's value depends on the actual workload composition, which remains untested beyond this read-heavy benchmark.

- **Should fundamentally different eviction families be explored?** All cycles optimize within bounded-window + frequency/recency. Would TTL-based expiration, statistical sampling, or simple FIFO with larger windows yield different latency/hit-rate curves?

## Per-policy notes

- **Bounded-window sequential search** (V001, Cycle 0002–0006): Robust foundation. Succeeds when implemented simply (min(capacity/4, 256) scan depth) with no additional logic. Attempts to layer conditional activation or dynamic allocation backfire. Best used as the baseline for all subsequent optimizations.

- **Frequency-level bucketing** (V002, Cycle 0002–0004): Effective at eliminating pathological scans and achieving hit-rate gains (0.6721→0.6904) at the cost of bucket-management overhead on the critical path. Works well when deferred (work pushed to put via promotion queues), hurts when bucket checks intrude on get. Meta-strategy: defer, don't inline.

- **Deferred promotion queues** (V002, Cycles 0004–0006): Reliably shifts work from get to put with measurable get-latency gains. Bulk flushing (all work at put, no drain checks on get) consistently beats incremental draining (checks on every get). The mechanism of deferral matters more than what metadata is deferred.

- **Adaptive/saturation-conditional strategies** (V003, Cycles 0003, 0005): Architecturally sound (apply expensive logic only when workload is under pressure), but liabilities on underutilized workloads where conditions never fire. Require empirical validation that thresholds are hit; mispredicted boundaries are pure overhead. Do not deploy without workload stress-testing.

- **Per-item metadata overhead** (frequency counters, operation counts, etc.): Consistently degrades latency on the critical path, especially if the metadata powers a search operation (min/max, sampling, etc.). Leaner metadata does not necessarily improve latency if it still requires expensive eviction-time operations. The fast path matters more than structure size.
