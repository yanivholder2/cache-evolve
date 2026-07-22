# Cross-Cycle Insights

## Recurring wins

- **Simple implementations outperform complex policies consistently**: Across all six cycles, ClassicLRU's straightforward O(n) vector operations on small buckets have remained more efficient than theoretically superior alternatives (O(1) timestamping, frequency tracking, deferred bookkeeping). This suggests cache locality, branch prediction, and compiler optimization favor simplicity over algorithmic sophistication.

- **Implementation-level optimization can deliver net gains when policy-level changes do not**: Cycle 0004's v003 (early-exit and cache-line alignment on 32×64 geometry) is the sole variant to achieve simultaneous hit-rate and latency improvement. This demonstrates the path forward involves micro-architectural tuning rather than policy innovation when policy is policy-invariant.

- **Zipfian hit-rate is policy-invariant over a broad range**: Hit-rate remains in the 0.661–0.665 band regardless of policy choice (ClassicLRU, FIFO, TimestampLRU, epoch-based, adaptive partitioning), confirming the workload's access distribution is well-matched to simple recency-based eviction and that policy diversity is not a fitness lever.

## Recurring failures

- **Policy changes uniformly fail to improve fitness**: Every policy innovation tested—LFU, FIFO, TimestampLRU, segment-based LRU, epoch-based distance tracking, adaptive hot/cold partitioning—either regressed latency or produced flat hit-rate. Across cycles 0001–0006, this pattern repeats without exception, suggesting the Zipfian workload is fundamentally well-matched to recency alone and that policy is exhausted as an optimization axis.

- **Per-hit bookkeeping overhead is structural and not addressable through policy isolation or indirection**: Cycles 0004–0006 test the hypothesis that moving bookkeeping to the eviction path (rare), partitioning structures (hot/cold), or deferring updates (epoch-based) could amortize cost. All three approaches add more overhead than they save, confirming that modern CPU memory hierarchies penalize any per-hit operation, regardless of architectural restructuring.

- **Asymptotic algorithm improvements invert in practice**: Cycles 0001–0005 repeatedly demonstrate that O(1) operations (timestamping, swapping, segment management) underperform O(n) operations (vector scanning) due to cache behavior, branch prediction, and memory-access latency. Theory-practice gaps are widest when constant factors are high; this workload and hardware expose them consistently.

- **Micro-optimizations do not generalize across geometries**: Cycle 0004's early-exit and cache-line alignment succeeded on 32×64 but failed to improve (or actively hurt) performance on 128×16, 64×32, and 16×128 in cycle 0005. Micro-optimization is geometry-sensitive; techniques tuned to one bucket size do not transfer, suggesting over-fitting rather than fundamental breakthrough.

- **Adding metadata and structural fields for algorithmic refinement backfires**: Cycle 0006's variants added per-item fields (epochs, frequency counters), expanding the Item struct, and incurred per-operation overhead to enable finer algorithmic control. Hit-rate remained flat (confirming policy is not a lever), while latency degraded. The cost of enabling clever ideas exceeds the benefit when the ideas themselves don't improve fitness.

## Open questions

- **Is cycle 0004 v003 a true fitness leader or a narrow local optimum?** It is the sole variant to improve both hit-rate and latency, but cycle 0005 showed the micro-optimizations do not generalize to other geometries. Should the next cycle pursue robustness validation (different workload distributions, cache sizes, contention scenarios) or accept it as geometry-specific and pivot to orthogonal approaches?

- **Have we exhausted the policy/geometry/micro-op optimization space for this workload context?** Six cycles of policy, geometry, and implementation tuning on a single-threaded, fixed-size cache with Zipfian access have yielded one tentative winner and many uniform failures. Is further refinement productive, or should we test robustness or pursue fundamentally different approaches (learned strategies, alternative data structures, multi-core efficiency)?

- **Why does every attempt to add structure incur more implementation cost than algorithmic gain?** Despite sound motivation (epoch tracking for deferred decisions, partitioning for isolation, prefetch awareness for cache coherency), all cycle 0006 variants failed. Is this a fundamental CPU property, or are we missing an architectural insight that could enable structured optimization?

- **Should the next cycle focus on validation of v003 or pivot to orthogonal directions?** Path A: empirically confirm v003's superiority across different workload distributions and cache sizes, and identify its failure modes. Path B: accept single-threaded Zipfian optimization as saturated and pursue learned insertion, cache-oblivious designs, cuckoo hashing, or multi-core efficiency. Recommend Path A first to establish a firm baseline before pivoting.

## Per-policy notes

- **ClassicLRU (baseline)**: 0.6611 hit-rate consistent across all six cycles; serves as a well-optimized reference point. No variant using a different policy has exceeded or matched its fitness. Simple O(n) vector operations on small (16-item) buckets exhibit superior cache behavior to all theoretically faster alternatives tested.

- **LFU (Least Frequently Used)**: Cycle 0001; regressed latency by 45%, flat hit-rate (0.6611). Frequency awareness does not reward Zipfian patterns where recency dominates. Dismissed after single test; not revisited.

- **FIFO (First-In-First-Out)**: Tested in cycles 0002–0004, 0006. Consistently maintained 0.6611 hit-rate, confirming hit-rate is policy-invariant on this workload. Per-hit latency regressed ~40% in cycle 0002 compared to ClassicLRU; FIFO's zero bookkeeping does not overcome scanning overhead. Policy integration failures in cycles 0003 (variant did not execute) and 0004 (FIFO with timestamp victim selection regressed 100–200 ns per op). Cycle 0006 FIFO component in adaptive hot/cold partitioning contributed to flat hit-rate and increased latency.

- **TimestampLRU**: Cycle 0002; pursued O(1) timestamp updates vs. ClassicLRU's O(n) vector ops. Latency regressed ~40% despite bookkeeping simplification, confirming timestamp overhead outweighs vector-scan savings. Never revisited after single test; pattern repeated in subsequent timestamp-based variants.

- **Segment-based LRU**: Cycle 0004; added per-operation segment metadata maintenance to bound eviction scan cost. Latency regressed 4×; hit-rate flat (0.661). Segment tracking on hit path proved too costly; bounding eviction scans is not worth the per-hit penalty.

- **Epoch-based distance tracking**: Cycle 0006 v001; attempted to defer eviction decisions to eviction time via access-distance epochs computed on every operation. Hit-rate flat (0.661), latency regressed due to per-hit epoch tracking and decay logic. Confirms that moving cost from eviction to hit path (or deferring via tracking fields) still adds net overhead.

- **Adaptive hot/cold partitioning with dual indices**: Cycle 0006 v003; most sophisticated policy tested; sound algorithmic fit for Zipfian workloads (small, distinct working set). Implemented frequency-based promotion, decay every 32 ops, and dual index vectors. Hit-rate flat (0.661–0.665), latency regressed due to dual-index search, frequency tracking, and promotion heuristic overhead. Confirms that implementation cost of enabling finer control exceeds benefit when control doesn't improve hit-rate.
