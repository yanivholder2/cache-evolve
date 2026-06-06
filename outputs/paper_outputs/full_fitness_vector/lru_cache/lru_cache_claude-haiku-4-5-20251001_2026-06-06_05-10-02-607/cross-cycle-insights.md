# Cross-Cycle Insights

## Recurring wins

- **Bounded-window eviction structures eliminate pathological O(N) scanning**: Across cycles, structural partitioning or pre-organization of eviction candidates (Cycle 0002 V001 bounded search, V002 bucket indexing, Cycle 0007 batched queue-flush) consistently prevent tail-latency explosions that plague unbounded linear scans. Window size is the primary hit-rate determinant; algorithm within the window is secondary.

- **Data structure choice dominates optimization more than algorithm choice**: Cycles 0002–0009 collectively demonstrate that the *mechanism* for candidate discovery (bounded search, bucket indexing, batched flushing) matters far more to latency than what signal drives selection (frequency, recency, LRU). Bucket indexing in particular is non-eliminable infrastructure for O(1) eviction, even when frequency tracking itself provides no hit-rate gain over simpler windowed strategies.

- **Batched algorithmic optimization breaks operation-shifting trade-offs**: Cycle 0007 V001's queue-flush coalescing and two-batch swapping first achieved simultaneous improvement in both get and put p99 (42% and 50% respectively), breaking the pattern from Cycles 0004–0006 where deferral always shifted cost between operations. This validates that the trade-off is not fundamental to deferral itself, but a property of naive flush mechanisms.

- **Hit rate reaches a plateau determined by window size and Zipfian skew**: Across Cycles 0004–0009, hit rate consistently stabilizes at 0.66–0.69 regardless of whether eviction uses frequency, recency, time-windowing, or pure LRU. This plateau is immutable once window size is set, suggesting the workload's Zipfian skew and window-size calibration exhaust all selectivity gains. Frequency tracking provides O(1) indexing infrastructure, not incremental selectivity beyond the window.

## Recurring failures

- **Asynchronous state approximation in adaptive decisions causes stale-feedback pathology**: Deferred saturation sampling (Cycle 0009 V001), lazy mode-switching allocation (Cycle 0005 V001), and operation-count approximation (Cycle 0009 V003) all regress latency or correctness because the delay between measurement and decision allows the cache to operate in suboptimal modes. The overhead of real-time feedback is consistently cheaper than the multiplicative cost of incorrect decisions acting on stale state.

- **Per-operation conditional logic overhead exceeds its benefit on light-load workloads**: Cycles 0008 V003 and 0009 V001 show that saturation checks or condition branches (division, comparison) intended to enable selective deferral cost more in median latency than their tail-latency gains recover. The workload's light utilization means expensive paths fire rarely, making even small per-operation overhead net-negative.

- **Removing frequency-bucket infrastructure exposes catastrophic O(N) eviction scanning**: Cycle 0009 V002 demonstrates that while frequency counting may not improve hit rate, the bucket indexing it enables is critical. Removing buckets to eliminate pointer-chasing overhead (which improved get p99 by 53%) immediately broke eviction candidate discovery, causing put p99 to spike 215% to 600 ns—a net catastrophe despite the hot-path improvement.

- **Conditional activation of expensive operations fails when trigger conditions don't fire**: Cycles 0007 V003, 0008 V001, and 0008 V003 show that infrastructure invested in saturation-triggered deferred promotion becomes pure overhead on underutilized workloads where saturation never materializes. The test workload remains below 70% utilization throughout; conditions tuned for 75% or 80% saturation contribute only bookkeeping cost.

- **Algorithmic optimizations are not orthogonal across contexts**: Cycle 0008 V002 attempted to separate batched queue-flush (which succeeded in Cycle 0007) from frequency tracking. Removing frequency while keeping batching caused cascading failures: hit rate dropped 4.8%, get p99 regressed 203%, and metadata overhead increased. Optimizations tightly coupled to their original context cannot be transferred cleanly.

- **Metadata overhead persists even when underutilized**: All saturation-aware and deferred-promotion variants (Cycles 0003–0008) added per-cache or per-node metadata (frequency counters, pending-update queues, saturation flags) that is always paid, even on light-load workloads where the expensive code paths never execute. Pre-allocation avoids pathological mode-switching costs but at the cost of permanent memory tax.

## Open questions

- **Can frequency-bucket infrastructure overhead be reduced without removing the structure?** Bucket indexing is non-eliminable, but the per-operation bookkeeping cost might be reducible via lock-free updates, batch processing of promotions, or approximate frequency maintenance. This would preserve O(1) eviction while reducing hot-path cost.

- **Does the optimization landscape change under sustained high saturation, write-heavy workloads, or longer traces?** Nearly all saturation-conditional variants assumed load profiles that never materialized on the tested read-heavy, light-saturation trace. Empirical measurement of actual saturation distribution across longer or synthetic adversarial workloads would reveal whether threshold-based strategies become valuable under different conditions.

- **What is the true hit-rate ceiling for this workload?** The 0.66–0.69 plateau across nine cycles across multiple eviction strategies suggests window size or Zipfian skew is the fundamental limit. Does increasing window size further validate this, or are there unexplored eviction signals (e.g., temporal locality beyond recency, predictive prefetching) that could unlock higher hit rates?

- **Can saturation tracking be computed nearly free?** Per-operation division and branch checks (Cycles 0008–0009) cost measurably more than they enable. Can saturation be tracked asynchronously via counter sampling, periodic polling, or amortized checks at allocation boundaries without incurring per-operation overhead?

- **Can architecture micro-optimizations (cache locality, pointer-chain reduction, vectorization) reduce bucket structure cost without removing it?** Cycle 0009 V002 showed 53% get p99 improvement by eliminating pointer-chasing; perhaps the bucket structure itself can be cache-optimized rather than removed.

- **Is the Cycle 0007 baseline locally optimal only for this workload, or does it generalize?** The baseline (batched queue-flush with frequency bucketing) succeeded on read-heavy, light-load conditions. Write-heavy synthetic traces, sustained high-saturation phases, or non-Zipfian access patterns would reveal whether the optimization landscape is fundamentally different.

## Per-policy notes

- **Frequency tracking via bucket indexing**: Cycles 0002, 0006, 0009 confirm this is load-bearing infrastructure for O(1) eviction. Frequency itself provides no hit-rate gain over window-size-calibrated LRU (Cycles 0003, 0009), but removing bucket indexing catastrophically breaks put latency via O(N) scanning. The frequency-bucket design is currently non-eliminable despite overhead.

- **Bounded-window LRU (fixed or adaptive)**: Cycles 0002–0003, 0007, 0009 show window-size tuning is the primary hit-rate lever. Window sizes from min(capacity/4, 256) to min(capacity/2, 512) all maintain hit rate parity (~0.6904–0.6652). Simple sequential scanning within the window is latency-efficient; selectivity gains beyond window size are marginal.

- **Deferred-promotion queue-flushing**: Cycles 0004–0007 show the mechanism (when flush occurs) dominates latency more than deferral itself. Pure-put-side flushing (Cycle 0006 V002) optimizes get but costs put; incremental draining (Cycle 0005 V002) spreads cost but introduces checks; batched coalescing (Cycle 0007 V001) achieves simultaneous improvement, suggesting algorithmic optimization is the key win.

- **Saturation-aware mode selection**: Cycles 0003, 0007–0009 show conditional activation is high-risk. When conditions fire (Cycle 0008 V003 at 45% threshold), per-operation checks dominate cost. When conditions don't fire (Cycle 0005 V003, 0008 V001 on light workload), infrastructure becomes pure overhead. Successful application requires either near-zero-cost checks or validated guarantee that conditions fire frequently.

- **Time-windowed and operation-count recency**: Cycle 0009 V002–V003 show these achieve hit-rate parity with frequency-based selection but offer no latency advantage. When recency tracking requires explicit search (min-operation-count scan), it introduces new latency bottlenecks. Implicit recency (via LRU ordering without tracking) appears preferable.

- **Dynamic mode-switching via lazy allocation**: Cycle 0005 V001 demonstrates pathological behavior when workload oscillates near saturation boundaries, triggering frequent allocation/deallocation cycles. Pre-allocation is necessary to avoid this; the permanent memory cost is acceptable as the price of avoiding allocation-time latency cliffs.
