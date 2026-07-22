# Cycle 0006 Insight

## Cohort outcome

This cohort tested three competing strategies to overcome the per-hit bookkeeping bottleneck identified in prior cycles: deferred eviction-time decision-making via access-distance epochs, micro-optimization through prefetch-aware reorganization, and adaptive hot/cold partitioning with dual indices. All three variants failed with either flat or regressed hit-rate and increased latency. This cycle represents the exhaustion of policy and structure-level optimization attempts within the current cache size and Zipfian workload context.

## Patterns

- **Eviction policy remains a non-lever across all strategies**: v001's epoch-based distance tracking, v003's hot/cold partitioning with frequency-based promotion, and v002's reorganization-aware reordering all employed fundamentally different policies, yet all produced flat or identical hit-rates (0.661–0.665, the Zipfian-indifferent baseline). Across cycles 0004–0006, this pattern repeats: hit-rate is invariant to policy selection. The conclusion that eviction strategy does not drive fitness on this workload is now confirmed beyond reasonable doubt.

- **Per-hit bookkeeping cost is structural and not addressable via indirection or deferral**: v001 attempted to move cost from hit path to eviction path (rare vs. common), but epoch tracking on every operation added more overhead than ClassicLRU's vector manipulation. v003 partitioned to isolate bookkeeping cost to a hot segment, but dual-index searches, frequency decay every 32 operations, and promotion logic added more latency than the savings from FIFO on cold items. The fundamental issue is not *where* bookkeeping happens, but that modern CPUs penalize any per-hit overhead, including tracking fields and conditional logic. Deferral or isolation does not eliminate the cost, merely distributes it.

- **Implementation complexity inverts good algorithmic ideas**: v003 was the most sophisticated variant algorithmically—adaptive hot/cold partitioning is a sound fit for Zipfian workloads where the working set is small and distinct. However, the implementation required dual index vectors, frequency tracking, decay loops, and promotion heuristics. The total per-operation cost of these mechanisms exceeded the benefit of using cheap FIFO on cold items. This mirrors cycle 0004's observation that policy innovation fails, but extends it: even well-reasoned structural changes fail when their implementation overhead is counted.

- **Metadata expansion adds cost without corresponding gain**: All three variants added per-item fields (epochs, frequency counters), expanding the Item struct by 20%. This overhead is paid on every operation but provides no improvement in hit-rate (which is policy-invariant). Cycle 0005 identified that compiler optimization narrows the margin for manual tuning; cycle 0006 shows that adding fields to enable algorithmic refinement backfires when the refinement itself doesn't improve fitness.

- **Cycle 0004's tentative success does not generalize outward**: v002 attempted to extend cycle 0004's v003 (32×64 + early-exit + cache-line alignment) by adding reorganization for prefetch coherency. Despite staying on the same geometry and preserving the existing optimizations, adding new logic (epoch tracking, in-place partitioning) disrupted the implicit ordering that early-exit relied on and added latency overhead that erased any prefetch gain. This suggests cycle 0004's v003 may have been over-fitted to a narrow point in the design space rather than a foundation for further progress.

## Open questions

- Is the cycle 0004 v003 (32×64 + micro-ops) truly a fitness leader relative to the full evolution history, or was it measurement noise from a narrow workload/cache-size window? Does it hold up under different access patterns, cache sizes, or contention scenarios?
- Have we exhausted the policy/geometry/micro-op space for single-threaded, fixed-size, Zipfian-workload caches? Is the baseline (128×16 ClassicLRU) Pareto-optimal at this level of analysis?
- Why does every attempt to add structure (tracking fields, index vectors, reorganization triggers) incur more cost than it saves, despite sound algorithmic motivation? Is this a fundamental property of CPU memory hierarchies, or are we missing a architectural insight?
- Should the next cycle pivot to orthogonal directions (learned strategies, alternative data structures, multi-core efficiency, robustness across workload distributions), or first empirically validate that cycle 0004 v003 is truly superior and worth defending?

## Continuation

Cycle 0006 has hit a ceiling: per-hit bookkeeping cost is structural, not addressable via policy, geometry, or micro-optimization. Every "clever" idea (deferral, partitioning, prefetch awareness) adds implementation cost that exceeds benefit. The next cycle must choose between two paths:

**Path A (Robustness & Validation)**: Empirically confirm whether cycle 0004's v003 is a true fitness leader by testing it across different workload distributions, cache sizes, and contention scenarios. If it holds, shift focus to understanding its failure modes and robustness boundaries rather than pursuing further optimization.

**Path B (Pivot to Orthogonal Directions)**: Accept that single-threaded, Zipfian, fixed-size cache optimization is saturated. Pursue fundamentally different approaches: learned insertion strategies (e.g., based on access patterns or key properties), cache-oblivious designs that break geometric dependence, cuckoo hashing or other probing strategies to reduce per-hit bookkeeping, or multi-core efficiency and contention reduction.

Recommend Path A first to establish a firm baseline, then Path B if robustness testing reveals brittleness. Do not continue policy or micro-optimization refinement—all three variants confirm the marginal return is zero or negative.
