# Cross-Cycle Insights

## Recurring wins

- **Structural data-structure choices dominate latency predictability over eviction strategy itself.** Bounded search windows (Cycle 0002 V001) and bucket indexing (Cycle 0002 V002) both eliminated O(N) latency spikes by constraining or pre-organizing candidate discovery, proving that *how* candidates are found is more critical than *which* candidate is selected. This pattern held consistently across Cycles 0002–0007, where every variant's latency behavior was determined by search structure, not eviction logic.

- **Removing checks from the critical path yields outsized latency returns.** Cycle 0006 V002 achieved 59% get p99 improvement by eliminating drain-trigger checks from get(), and Cycle 0007 V001 achieved balanced tail improvements through algorithmic optimization of the flush mechanism rather than distributing work across both operations. Both demonstrate that critical-path minimalism compounds over millions of operations.

- **Batching and coalescing reduce peak operation cost without inherent operation-shifting.** Cycle 0007 V001's batched queue-flush mechanism achieved simultaneous improvement of both get p99 (42%) and put p99 (50%) for the first time, breaking the operation-shifting trade-off pattern from Cycles 0004–0006. This shows that deferred-work trade-offs are tunable through algorithmic optimization of the flush mechanism, not structural inevitabilities.

- **Workload-aware conditional activation outperforms uniform strategies when conditions fire.** Cycle 0003 V003 achieved 34% get p99 reduction by activating frequency bucketing only above 75% saturation, and Cycle 0002 V001's bounded window maintained flat latencies where V002's bucketing added overhead. However, subsequent cycles showed this pattern only holds when the conditions actually trigger.

## Recurring failures

- **Deferring work merely shifts cost between operations rather than eliminating it.** Cycles 0004–0007 consistently show that deferred-work strategies move computational burden from get to put (or vice versa) but do not reduce total work. Cycle 0004 V002, Cycle 0005 V001, and Cycle 0006 V001 all demonstrate cost shifting; Cycle 0007 V001 broke this pattern only through algorithmic (flush-mechanism) optimization, not deferral structure.

- **Sampling-based and decision-deferral logic add bookkeeping overhead that outweighs benefit on light-load workloads.** Cycle 0004 V001's periodic sampling (counter increment and modulo every 16 operations) cost more than the lightweight decision logic being sampled. Counter increments and conditional checks on the fast path are consistently more expensive than executing the simple decision outright on underutilized caches.

- **Dynamic allocation for mode-switching creates latency cliffs rather than smooth transitions.** Cycle 0005 V001's lazy bucket initialization via `Option<Box<>>` with oscillating saturation boundaries caused 40× get p99 spikes during allocation/deallocation cycles. Pre-allocated structures, while permanently occupying memory, avoid the indirection and allocation overhead that ripple through pointer-chasing code paths.

- **Per-operation frequency tracking adds overhead without proportional hit-rate gain after the bounded-window baseline.** Cycles 0004–0006 show that frequency counters, multi-level ranking, and recency tracking all increased latency cost (Cycle 0004 V003: 2.8× put p99 spike; Cycle 0006 V003: latency cliff during search) while hit rate remained stable at ~0.69 across all variants. Frequency selectivity is already captured by bounded-window size alone.

- **Conditional logic protecting correctness must activate universally or preserve baseline semantics when inactive.** Cycle 0007 V003 deferred LRU promotions based on an 80% saturation threshold that never fired, breaking the LRU ordering invariant and causing hit rate to drop 9.4% (0.6904→0.6255). Deferring correctness-critical operations without universal activation invalidates cache semantics.

## Open questions

- **Can batched queue-flush mechanisms reduce their median-latency cost while preserving tail improvements?** Cycle 0007 V001 achieved balanced p99 improvements (get/put both ~45%) but at 20% get p50 regression. Systematic tuning of batch size, swap frequency, and drain thresholds may reduce this overhead without surrendering tail gains.

- **Do tail-latency and hit-rate optimizations hold under sustained high saturation and write-heavy workloads?** All testing to date has been on light-load, read-heavy traces where saturation remains below 70% and deferred-promotion opportunities are minimal. Will V001 (Cycle 0007) and conditional strategies (Cycle 0003) maintain improvements when saturation is continuous or write ratio increases?

- **What is the true hit-rate ceiling for this workload and window size?** Cycles 0004–0007 show hit rate plateaued at ~0.66–0.69 regardless of eviction strategy (frequency, recency, random sampling, FIFO). Is the Zipfian skew or window size already optimal, or would adversarial access patterns or broader frequency distributions reveal opportunities for selective eviction?

- **Can saturation-aware mode switching be salvaged with corrected thresholds or altered semantics?** Cycle 0005 V003's architecture (simple mode below 75%, bucketing above) was sound but never triggered on the tested workload. Can trace analysis or longer stress tests identify workloads where saturation genuinely crosses thresholds, validating the design? Alternatively, can mode-conditional LRU promotions preserve correctness at all saturation levels?

- **Is eviction-candidate search cost a memory-hierarchy or algorithmic bottleneck?** Cycle 0006 V003 showed that simplified recency tracking (eliminating frequency counters) still introduced latency cliffs during window scans. Is the cost driven by cache misses on small-window scans, pointer chasing, or the min-search algorithm itself? This affects whether different search structures (e.g., approximate recency or bloom filters) would help.

## Per-policy notes

- **Frequency-based eviction (Cycles 0001–0006):** Initial strategy achieved hit-rate gains (0.6721→0.6904 in Cycle 0002) but per-operation frequency tracking overhead (Cycles 0004–0006) made the latency cost substantial. Bucket indexing (Cycle 0002 V002) avoided pathological latency but added get p99 overhead (67→147 ns). Hit rate improvements plateaued after Cycle 0002; further frequency-tracking sophistication (multi-level ranking, random sampling) did not improve hit rate while uniformly increasing latency cost. Frequency selectivity appears already captured by bounded-window size.

- **Zone/spatial partitioning (Cycle 0001 V001):** Pre-partitioning eviction zones avoided latency spikes but saw no hit-rate benefit. Structural simplicity does not compensate for reduced selectivity; subsequent cycles abandoned pure spatial approaches in favor of bounded-window search.

- **Bounded-window sequential search (Cycles 0002+):** Emerged as a stable foundation, used in every subsequent baseline. Achieves ~0.69 hit rate and predictable latency without pathological worst-case behavior. Remains a low-overhead alternative to frequency tracking, suggesting it already captures most selectivity value for Zipfian workloads. Window size trades off hit rate for eviction cost; larger windows (Cycle 0007 V002) maintained hit rate.

- **Deferred promotion (Cycles 0004–0007):** Trades get latency improvement for put latency cost (Cycle 0004: 23% get p99 gain at 56% put p99 cost). Algorithmic optimization of queue flushing (Cycle 0007 V001) can reduce this trade-off asymmetry through batching and coalescing, achieving balanced improvements. Deferred work is viable when the flush mechanism is explicitly optimized; naive deferral just shifts cost.

- **Adaptive/conditional strategies (Cycles 0003, 0005, 0007):** V003 in Cycle 0003 (adaptive bucketing) won by activating expensive logic only when saturation exceeded 75%, achieving 34% get p99 improvement on an underutilized workload. However, subsequent attempts (Cycle 0005 V001: dynamic allocation; Cycle 0007 V003: deferred promotions) show that conditional activation is valuable only when thresholds fire reliably and semantics are preserved at all activation states. Unmet conditions and untested branches are reliability hazards.

- **FIFO with larger windows (Cycle 0007 V002):** Confirmed that hit rate is primarily window-size dependent and frequency tracking is eliminable metadata overhead. Hit rate remained flat (0.6904) when window doubled, validating that selectivity comes from breadth, not tracking sophistication. However, latency improvements from eliminating frequency overhead were marginal, suggesting memory-hierarchy effects or branching dominate frequency-counter cost on this light-load workload.
