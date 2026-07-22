# Cycle 0008 Insight

## Cohort outcome

This cohort tested three orthogonal refinements to the prior winner (cycle 0004 v003): geometry scaling to validate generalizability, tail-preferential eviction to improve hit-rate without overhead, and learned insertion positioning based on access history. All three variants failed to improve net fitness—geometry scaling severely degraded latency, naive recency tracking reduced hit-rate, and the learned approach achieved modest hit-rate gain at prohibitive latency cost (40–50% increase).

## Patterns

- **Cycle 0004 v003's optimizations are finely tuned to 32×64 geometry, not generalizable**: v001's attempt to change bucket geometry from 32×64 to 64×32 (narrower buckets) caused severe latency regression—get p99 doubled, put p50 tripled. Early-exit optimization's effectiveness is inseparable from wide buckets; the cost-benefit of scanning fewer items diminishes at smaller bucket sizes. This confirms that the prior winner is a localized optimum rather than a foundation for proportional scaling.

- **Simple recency heuristics fail to capture Zipfian reuse distances**: v002's tail-preferential eviction with 64-operation recency reset intervals achieved flat latency but reduced hit-rate significantly (0.664 → 0.6137). The coarse-grained recency flag is not a sufficient proxy for the frequency/reuse-distance patterns in the Zipfian workload. Position-based LRU ordering implicitly captures these patterns better than explicit recency tracking.

- **Learned insertion positioning has algorithmic merit but faces implementation overhead barriers**: v003 improved hit-rate by +0.43% through predictive bias based on recent access history, proving that algorithmic adaptivity (learning from temporal locality shifts) can steer cache behavior positively. However, polling a 16-element rolling buffer on every operation incurred 40–50% median latency overhead, making the hit-rate gain Pareto-dominated by latency cost. The idea is sound; the implementation is inefficient.

- **No variant achieved net fitness improvement; prior winner remains unbeaten**: Despite three distinct approaches—scaling, eviction-policy refinement, and learned insertion—all produced either flat or regressed fitness. Cycle 0004 v003 retains the best fitness across all eight cycles and appears to be a genuine local optimum in the current design space.

- **Metadata expansion and per-operation bookkeeping remain liabilities**: v002 added 8-byte per-item boolean fields; v003 added cache-struct overhead. Both incurred costs without commensurate gain. This reinforces cycles 0006–0007 findings that any structural change to enable algorithmic refinement tends to cost more than it saves, especially on latency-sensitive paths.

## Open questions

- Is v003's learned insertion approach viable with an amortized or vectorized implementation that reduces per-operation polling overhead to acceptable levels?
- Does cycle 0004 v003 remain optimal across different workload distributions (non-Zipfian, multi-modal access patterns) and cache sizes, or is its fitness edge workload-specific?
- Have we reached the ceiling of single-threaded, fixed-size cache optimization, or are there unexamined algorithmic directions (e.g., adaptive learned thresholds for eviction, cache-oblivious probe ordering)?
- Is the 0.43% hit-rate improvement from learned insertion worth pursuing if overhead can be reduced, or does the Zipfian workload's inherent hit-rate ceiling (empirically ~0.7 across variants) limit upside?

## Continuation

Cycle 0008 closes the refinement and scaling paths for cycle 0004 v003: geometry scaling fails, simple eviction-bias refinement fails, and learned insertion trades unacceptable latency for marginal hit-rate. The cohort confirms that the prior winner is a precise local optimum that resists incremental improvement.

**Recommend two parallel investigations for the next cycle:**

1. **Robustness validation of cycle 0004 v003**: Test its fitness across non-Zipfian workload distributions (uniform, multi-modal, temporal locality shifts), varying cache sizes (512–8192 entries), and contention scenarios. If it remains superior, the design space is saturated and focus should shift to implementation stability and generalization; if it regresses, the next cycle can pursue alternatives with confidence.

2. **Lower-overhead learned insertion**: If v003's algorithmic idea of predictive biasing is to be explored further, redesign to amortize buffering cost (e.g., update insertion bias every N insertion operations rather than every operation, or vectorize history tracking). A 0.43% hit-rate gain is modest but not negligible if overhead can be halved or quartered.

If neither direction yields improvement, the next cycle should pivot decisively to orthogonal approaches: alternative data structures (cuckoo hashing, Robin Hood variants with amortization), cache-oblivious designs, multi-core efficiency, or learned indexing strategies that operate at insertion time rather than in-path polling.
