# Cross-Cycle Insights

## Recurring wins

- **Implementation micro-optimization over policy change**: Cycle 0004 v003 (early-exit + cache-line alignment on 32×64 geometry) remains the only variant to achieve net fitness improvement. It succeeded by optimizing the hot path (hit detection) rather than changing eviction policy or bucket structure. All policy variants (LFU, FIFO, timestamp-based, segment-based, frequency-aware) across cycles 0001–0009 failed to improve fitness.

- **Simplicity survives complexity**: ClassicLRU's O(n) vector manipulation consistently outperforms asymptotically superior alternatives (O(1) timestamp updates, zero-cost FIFO tracking, probabilistic frequency counters). Every attempt to replace or optimize the core mechanism either regressed or produced null results, suggesting the baseline is finely tuned to CPU memory hierarchies and compiler optimization patterns.

- **Geometry 32×64 with focused optimization**: Cycle 0004 v003's wide-bucket geometry (32 items per bucket) combined with early-exit and cache-line alignment remains the fitness leader across all cycles. This pairing addressed Cycle 0003's latency crisis by optimizing implementation details rather than changing policy, establishing that wide buckets are viable if latency is engineered carefully.

- **Characterization-based insertion bias has algorithmic merit**: Cycle 0009 v003 achieved +8.1% hit-rate improvement through workload characterization and adaptive insertion bias, the largest hit-rate gain observed across any single variant. This demonstrates that algorithmic adaptivity can steer cache behavior positively, even if latency cost in the current framework is prohibitive.

## Recurring failures

- **Policy innovation as a lever**: Across cycles 0001–0010, eviction policy swaps (LFU, FIFO, TimestampLRU, segment-based LRU, FIFO with victim tracking, epoch-based distance, hot/cold partitioning, probabilistic frequency, recency-based bias) produced flat or regressed hit-rate on the Zipfian workload. Hit-rate is policy-invariant on this access pattern; the working set is small and distinct enough that any policy achieving recency approximation performs equivalently. Policy change is not a lever in this design space.

- **Per-hit bookkeeping and metadata expansion**: Every variant that added tracking fields, frequency counters, index vectors, or per-operation logic regressed latency—some by 40%, others by 2–3×. Across cycles 0006–0010, added metadata ranges from 1 byte (probabilistic counters) to 56 bytes (characterization state), but all incur unacceptable per-operation cost. No strategy (amortization, quantization, batching, deferred updates) successfully reduced overhead without sacrificing algorithmic precision or hit-rate.

- **Amortization consolidates rather than smooths latency**: Cycles 0009–0010 attempted to defer overhead by batching updates every N operations. Result: p50 latency remained flat, but p99 latency spiked (doubled to quintupled), indicating that consolidating bookkeeping cost into intermittent expensive operations fails the latency constraint. Per-operation cost is unavoidable; deferral only creates load imbalance.

- **Micro-optimization does not generalize across geometries**: Cycle 0005 applied Cycle 0004 v003's early-exit and cache-line alignment techniques to baseline 128×16, 64×32, and 16×128 geometries. All regressed or produced null results. Early-exit optimization's cost-benefit is inseparable from 32×64 bucket scanning patterns; narrower buckets see less benefit, wider buckets exceed the technique's design threshold.

- **Geometry scaling from the winner breaks**: Cycles 0003, 0005, 0008 attempted to scale or adjust bucket geometry away from the Cycle 0004 v003 baseline (32×64). Every attempt regressed latency or hit-rate. The geometry is not a general optimum but a narrow local optimum finely tuned to the current workload and hardware.

- **Orthogonal optimization creates cross-path regression**: Cycle 0004 showed that optimizing the eviction path (rare) can backfire when it disrupts the hit path (common). Cycles 0005, 0006, and 0007 confirmed this: structural changes to eviction, capacity management, or probe variance consistently regressed get latency or broke compiler optimizations that the baseline relied on. Orthogonal improvements have unintended consequences.

## Open questions

- **Is Cycle 0004 v003 (or Cycle 0006 v003) the true fitness leader, or a workload-specific local optimum?** Neither variant has been validated across non-Zipfian workload distributions (uniform, multi-modal, temporal shifts), varying cache sizes (512–8192 entries), or contention scenarios. Robustness testing is needed to establish whether these leaders generalize or are narrow optima that break under different conditions.

- **Why does amortization consolidate overhead into spikes rather than smooth it?** Cycles 0009–0010 identified that per-operation check cost (counter increment, modulo test) and deferred work batching create intermittent expensive operations. Is there a batching or vectorization strategy that achieves true amortization without tail-latency degradation?

- **Why does ClassicLRU's O(n) vector manipulation outperform O(1) timestamp updates?** This theory-practice inversion repeats across multiple cycles. Is the answer cache-line coherency, branch prediction stability, compiler loop optimization, or memory access patterns specific to 16-item vectors?

- **Have all reasonable directions within single-threaded, Zipfian, fixed-size optimization been exhausted?** Cycles 0001–0010 covered policy change, geometry tuning, micro-optimization, metadata expansion, amortization, and characterization-based learning. All produced flat or regressed fitness. Are cache-oblivious designs, alternative data structures (cuckoo hashing, Robin Hood), learned indexing, or multi-core efficiency the only unexamined orthogonal directions?

- **Is Cycle 0009 v003's +8.1% hit-rate actually achievable at acceptable latency through different implementation strategies?** Cycle 0010 showed that type quantization, batched tracking, and early termination all destroy either precision or characterization depth. Can vectorization or pre-computed bias classes reduce per-operation overhead without sacrificing algorithmic gain?

## Per-policy notes

- **ClassicLRU (baseline, Cycle 0002+)**: Consistently stable hit-rate (0.6611) and latency. O(n) vector operations on 16-item buckets have unusually good cache locality or compiler optimization. No variant that attempted to replace it achieved fitness improvement. Appears finely tuned to the hardware.

- **LFU (Cycle 0001)**: Degraded both latency (45% slowdown) and hit-rate (0.6568). Frequency awareness does not help Zipfian workload where recent access is a better eviction signal than global frequency. Per-hit bookkeeping cost outweighed any benefit.

- **TimestampLRU (Cycle 0002)**: Maintained hit-rate (0.6611) but regressed latency (40%). O(1) timestamp updates traded off against ClassicLRU's vector operations. Asymptotic complexity advantage inverted under CPU memory hierarchies.

- **FIFO (Cycle 0002, Cycle 0003)**: Hit-rate flat (0.6611) despite zero per-hit recency tracking, proving policy is orthogonal to hit-rate on Zipfian workload. Designed variant in Cycle 0004 v001 to add victim tracking but regressed latency (100–200ns overhead), revealing that FIFO's zero-cost advantage disappears when any tracking is added.

- **Segment-based LRU (Cycle 0004 v002)**: 4× latency regression despite sound algorithmic design. Optimizing rare eviction paths (segment rebalancing) at cost of per-hit overhead is a losing trade-off; the hit path dominates execution time.

- **Hot/cold partitioning with frequency decay (Cycle 0006 v003)**: Hit-rate improved (0.6988) through adaptive frequency-based promotion within dual indices, but dual-index searches, decay loops, and promotion logic added latency that exceeded the savings. Metadata expansion and per-operation bookkeeping cost more than the algorithmic benefit.

- **Characterization-based insertion bias (Cycle 0009 v003, Cycle 0010)**: Achieved +8.1% hit-rate improvement by analyzing access patterns during a 512-operation phase and biasing insertion position. Algorithmic merit confirmed, but 2–3× latency cost is prohibitive. Cycle 0010 attempts to reduce overhead through type quantization, batching, or early termination all traded precision or characterization depth for latency, leaving net fitness flat.

- **Recency-based eviction bias (Cycle 0008 v002)**: Flat latency but hit-rate collapsed (0.664 → 0.6137). Coarse-grained recency flags (64-operation intervals) insufficient to capture Zipfian reuse distances. Simple position-based LRU ordering implicitly captures these patterns better than explicit recency tracking.

- **Probabilistic frequency tracking (Cycle 0009 v002)**: Flat hit-rate despite minimal memory overhead (1 byte per item using Morris counter). Frequency awareness does not improve hit-rate on this workload; any frequency-aware logic (seed computation, masking, conditional branch) incurs ~31% latency cost with no benefit.
