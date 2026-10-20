# Cross-Cycle Insights

## Recurring wins

- **ClassicLRU is finely tuned, not just correct**: The baseline maintains consistent 0.6611 hit-rate and consistently outperforms all variants on latency despite their theoretical advantages. It appears optimized at the cache-locality and micro-architectural level for the benchmark scale and Zipfian workload.
- **Hit-rate robustness to policy semantics**: Across both cycles, hit-rate proved insensitive to eviction policy details—recency, frequency, timestamps, and even FIFO all achieved ~0.66 hit-rate on Zipfian access. This suggests the workload's statistical structure alone (heavy-tailed access) sustains high hit-rate regardless of policy.
- **Small bucket geometry (128×16) outperforms larger variants**: When tested directly, wider buckets (64×32) improved hit-rate modestly but at latency cost that no variant could absorb. The current geometry appears locally optimal for this workload and scale.

## Recurring failures

- **Asymptotic complexity improvements fail in practice**: Both O(1) removal (Swap, cycle 0001) and O(1) timestamping (TimestampLRU, cycle 0002) underperformed ClassicLRU's O(n) vector operations, revealing that constant factors—cache coherency, branch prediction, swap chain overhead—dominate real performance at this benchmark scale.
- **Any added bookkeeping incurs measurable overhead**: Whether frequency counters, timestamp updates, or longer bucket lists, all variants slowed by 40–45% on per-operation latency compared to the baseline. No optimization at the policy level recovered this cost.
- **Policy refinements miss the real constraint**: Hit-rate already saturates on this workload; improvements come only from geometry changes that increase latency. Eviction policy is not the bottleneck for Zipfian access.

## Open questions

- **Why is ClassicLRU's O(n) vector manipulation so fast?** Micro-profiling is needed to identify whether cache line coherency, compiler loop unrolling, branch prediction patterns, or memory prefetching explain the margin over O(1) alternatives.
- **Is 128×16 globally optimal or locally optimal?** Does the current geometry hold across different workload distributions (non-Zipfian), cache sizes, or multi-core contention scenarios?
- **Can implementation-level refinements (vectorization, prefetching, cache-aligned layouts) recover the benefits of wider buckets without latency regression?** Geometry changes are promising for hit-rate but must be paired with micro-architectural optimization to be viable.

## Per-policy notes

- **Frequency-aware (LFU)**: Overhead of maintaining counts not justified; Zipfian workload doesn't reward frequency over recency. Hit-rate no better than baseline.
- **FIFO**: Surprisingly effective, maintaining baseline hit-rate despite zero recency awareness. Suggests policy semantics matter less than cache geometry for this workload.
- **TimestampLRU**: O(1) per-access cost of timestamp updates still slower than O(n) vector operations. Confirms that bookkeeping method orthogonal to hit-rate, but implementation cost is tangible.
- **Swap (O(1) removal)**: Asymptotic advantage (O(1) vs O(n)) completely inverted at benchmark scale. Swap chain overhead and cache coherency penalties dominate.
- **Bucket repartitioning (16→32 items, then 128×16→64×32)**: Wider buckets improve hit-rate (+0.24%) but sequential intra-bucket scans become costlier. Trade-off is unfavorable for latency-bounded workloads.
- **ClassicLRU**: The reference implementation appears to be a local maximum—small buckets with vector-based recency tracking, optimized for cache locality. Difficult to improve without understanding its micro-architectural advantages.
