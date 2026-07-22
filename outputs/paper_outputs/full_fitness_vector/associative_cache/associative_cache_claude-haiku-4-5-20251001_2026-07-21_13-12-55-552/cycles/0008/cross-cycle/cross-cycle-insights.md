# Cross-Cycle Insights

## Recurring wins

- **ClassicLRU baseline remains unbeaten across eight cycles**: Despite continuous attempts to improve via policy, geometry, or micro-optimization, no variant has decisively displaced the baseline. Hit-rate is invariant to policy choice on Zipfian workloads (0.66–0.70 range), making latency the primary fitness dimension—and ClassicLRU's simple O(n) vector operations consistently outperform theoretically superior O(1) alternatives.

- **Implementation-level optimization succeeds where policy fails**: Cycle 0004 v003 (32×64 geometry with early-exit scanning and cache-line alignment) is the only variant to achieve clear fitness improvement by optimizing the hot path rather than changing policy or bookkeeping. This success stands in contrast to every policy innovation (LFU, FIFO variants, segment-based LRU, Robin Hood) which uniformly regressed or produced Pareto-dominated trades.

- **Latency overhead of added bookkeeping is structural and unavoidable**: Across cycles 0004–0008, every attempt to add tracking fields, metadata, or conditional logic for algorithmic refinement incurred per-operation cost that exceeded benefit. Deferral (cycle 0006 v001), partitioning (cycle 0006 v003, cycle 0007 v001), and batching proposals all underestimated the cost of CPU memory-hierarchy penalties and compiler side effects.

## Recurring failures

- **Per-hit bookkeeping cost cannot be eliminated via policy isolation or deferral**: Cycles 0004–0006 tested moving overhead from hit path to eviction path (rare vs. common), partitioning bookkeeping to subsets, and amortizing via epochs. All failed—epoch tracking added more cost than ClassicLRU's vector manipulation, dual-index partitioning incurred search overhead, and promotion logic on every operation defeated the goal. The problem is not *where* bookkeeping happens, but that modern CPUs penalize any per-hit overhead, including tracking fields and conditional checks.

- **Frequency-aware and complex eviction policies regress across all attempts**: LFU (cycles 0001, 0006), segment-based LRU (cycle 0004 v002), hot/cold partitioning (cycle 0006 v003), Robin Hood insertion (cycle 0007 v003), and learned insertion (cycle 0008 v003) all either maintained flat hit-rate or improved it at prohibitive latency cost (2–4× regression or near-doubling). The Zipfian workload's hit-rate ceiling appears independent of eviction policy; position-based LRU order implicitly captures reuse patterns better than any explicit tracking.

- **Micro-optimizations do not generalize across geometric configurations**: Cycle 0004 v003's success with early-exit and cache-line alignment on 32×64 geometry failed to transfer when applied to 128×16 (cycle 0005 v001, latency regressed 45%), 64×32 (cycle 0005 v002, no gain), or 16×128 (cycle 0005 v003, latency tripled). The micro-optimization was finely tuned to a specific bucket-scan profile and inverts at geometric extremes, confirming that implementation-level gains are not universal levers.

- **Theory-practice complexity inversions are consistent**: Across cycles, asymptotically superior approaches (O(1) timestamping vs. O(n) vector ops, minimizing per-hit bookkeeping, reducing probe distances) consistently underperformed due to cache-line coherency, branch prediction, and compiler layout effects. The constant factors and micro-architectural costs dominate practical performance, making asymptotic complexity a weak predictor of real-world latency.

## Open questions

- **Is cycle 0004 v003 truly a fitness leader that generalizes, or a narrow local optimum?** It achieved the only clear improvement but cycles 0005 and 0008 suggest its optimizations are specific to 32×64 geometry and the Zipfian benchmark. Validation across non-Zipfian workloads, varying cache sizes, and contention scenarios is needed to establish whether it is a foundation for further progress or an over-fitted point.

- **Why does adding structure and tracking (fields, indices, reorganization) consistently cost more than it saves?** Cycles 0006–0007 show that sound algorithmic ideas (adaptive partitioning, epoch tracking, prefetch awareness) incur implementation overhead that exceeds benefit. Is this a fundamental property of modern CPU memory hierarchies, or a missing architectural insight (e.g., vectorization, prefetching, cache-oblivious design) that could reclaim the overhead?

- **Has single-threaded, fixed-size, Zipfian-workload cache optimization been exhausted at the policy/geometry/micro-op level?** Eight cycles of refinement show hit-rate plateauing at 0.66–0.70 and latency-variance floor established by CPU micro-architecture. Do orthogonal approaches (learned insertion heuristics, cuckoo hashing, cache-oblivious designs, multi-core contention reduction) offer upside, or is the design space fundamentally saturated for this problem class?

- **Can learned insertion biasing (cycle 0008 v003's +0.43% hit-rate) be made practical through lower-overhead amortization (e.g., update bias every N operations, vectorized history tracking)?** The algorithmic merit is clear, but per-operation polling cost is prohibitive. Redesign for amortization might recover upside without latency penalty.

## Per-policy notes

- **ClassicLRU (position-based recency LRU)**: Baseline throughout, remains unbeaten and most latency-efficient. Hit-rate varies with geometry but is invariant to policy refinement (0.6611 base, up to 0.6988 with geometry tuning). No variant decisively outperformed it; cycle 0004 v003 matched its hit-rate while optimizing latency variance on 32×64 geometry.

- **Frequency-aware policies (LFU, frequency-decay, adaptive hot/cold)**: Tested in cycles 0001, 0006, and 0007. Uniformly failed—LFU (cycle 0001) and hot/cold partitioning (cycle 0006 v003) maintained 0.66 hit-rate despite added bookkeeping. Frequency decay loops and promotion logic added latency without proportional gain. Zipfian access patterns render frequency tracking redundant; position-based order suffices.

- **FIFO (first-in-first-out eviction)**: Cycles 0002, 0003, attempted as zero-per-hit-overhead policy. Maintained baseline hit-rate (0.6611) but when implemented with victim-selection optimization (cycle 0004 v001), required O(n) bucket scan that added 2× latency regression. Policy itself is sound but implementation to avoid per-hit cost is non-trivial; naive victim selection defeats the goal.

- **Timestamp-based LRU (TimestampLRU)**: Cycle 0002, pursued O(1) update vs. ClassicLRU's O(n) vector ops. Regressed latency 40–45% despite theoretical advantage, confirming constant-factor and cache-hierarchy dominance over asymptotic complexity.

- **Segment-based and adaptive policies**: Cycles 0004 v002 (segment-based LRU), 0006 v003 (hot/cold partitioning), 0007 v001 (reduced-partition adaptive). All added per-operation bookkeeping (frequency tracking, promotion logic, dual indices) that cost more than savings from rare eviction optimization or partition-specific policies. Algorithmic elegance does not survive implementation overhead.

- **Learned and probing-based policies (Robin Hood insertion, learned insertion positioning)**: Cycles 0007 v003, 0008 v003. Robin Hood achieved +8.6% hit-rate at near-doubling latency (Pareto-dominated). Learned insertion achieved +0.43% hit-rate with 40–50% latency overhead—algorithmic merit (predictive biasing from access history) is evident, but overhead is prohibitive without lower-cost amortization strategy.
