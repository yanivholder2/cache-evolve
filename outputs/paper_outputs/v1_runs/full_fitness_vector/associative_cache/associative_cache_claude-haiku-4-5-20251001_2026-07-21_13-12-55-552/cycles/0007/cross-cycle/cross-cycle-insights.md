# Cross-Cycle Insights

## Recurring wins

- **ClassicLRU's policy stability on Zipfian workloads**: Hit-rate (~0.6611) remains invariant across five distinct eviction policy changes (LFU, Timestamp, FIFO, Segment-based, Epoch-based), demonstrating that recency-based eviction already captures Zipfian access patterns. Policy innovation is not a fitness lever for this workload.

- **Micro-optimization success when targeting the hit path**: Cycle 0004 v003 (32×64 geometry with early-exit and cache-line alignment) achieved the only net fitness improvement across all seven cycles—preserving hit-rate while halving tail latencies. The lesson: optimize the common case (hits), not rare cases (evictions).

## Recurring failures

- **Policy changes uniformly increase per-hit bookkeeping overhead**: Every eviction policy variant—LFU, Timestamp, FIFO with victim selection, Segment-based LRU, Epoch-based distance tracking, Hot/cold partitioning, Robin Hood hashing—regressed latency by 40% to 4× relative to ClassicLRU. Theoretical asymptotic gains (O(1) operations, reduced scan depth, improved distribution) evaporate under modern CPU memory hierarchies and per-operation costs.

- **Geometry variations produce latency regressions unless finely micro-optimized**: Narrower (256×8), wider (32×64, 64×32, 16×128), and denser (64×32) bucket configurations all initially degraded latency. Hit-rate improvements (when achieved) came at 40–300% latency costs, leaving fitness unchanged or worse. Geometry-driven hit-rate gains require surgical implementation optimization (early-exit, cache alignment) to be viable.

- **Micro-optimizations do not generalize across geometric configurations**: Cycle 0005 definitively showed that early-exit and cache-line alignment—successful on 32×64—hurt performance on 128×16 (get p50 +45%, p99 doubled) and showed zero benefit on 64×32. Micro-tuning is configuration-specific, not a universal lever. Ultra-wide 16×128 degraded 3× under the same optimizations.

- **Structural overhead exceeds algorithmic benefit**: Adding tracking fields (epochs, frequency counters, partition indices), expanding metadata by 20%, or reorganizing code for prefetch coherency—despite sound algorithmic motivation—consistently incurs more cost on hits than saved on rare evictions. Metadata bloat is pure loss when hit-rate doesn't improve.

- **Compiler optimization narrows the tuning margin**: Unintended code-structure changes (cycle 0007 v002's eviction loop refactoring disrupted hit-path inlining and regressed get latency). Compiler behavior is opaque and resistant to manual micro-tuning; attempts to improve one path often degrade another.

## Open questions

- **Is cycle 0004 v003 (32×64 + micro-ops) a true fitness leader or a narrow local optimum?** Cycles 0005–0007 failed to extend its success, and it has not been validated across different workload distributions or cache sizes. Does it hold up under non-Zipfian access patterns, larger/smaller caches, or contention?

- **Why does ClassicLRU's O(n) vector manipulation (remove + append on 16-item buckets) outperform O(1) alternatives (Timestamp, FIFO)?** Cache coherency, branch prediction, compiler loop unrolling, or memory layout? Understanding this micro-architectural fact could inform future policy design.

- **Has single-threaded, Zipfian, fixed-size cache optimization saturated?** Cycles 0006–0007 show zero progress from policy, geometry, and micro-optimization levers. Are all degrees of freedom exhausted, or are there unexplored approaches (learned strategies, alternative data structures, adaptive algorithms)?

- **When does ClassicLRU break?** No variant has outperformed the baseline on fitness. Does the baseline fail under different workload distributions (non-Zipfian, uniform, adversarial), multi-core contention, or larger cache sizes? Or is it robust enough to dismiss further optimization?

## Per-policy notes

- **ClassicLRU (baseline)**: Consistently achieves ~0.6611 hit-rate across all test configurations and policy variations. Latency baseline is the performance ceiling—no variant has improved it. The O(n) vector manipulation on hits is unexpectedly efficient; attempts to replace it with O(1) alternatives all regressed.

- **Frequency-based policies (LFU, Hot/cold partitioning, Epoch-based)**: Frequency tracking adds per-hit overhead (bookkeeping fields, decay loops, promotion logic) without improving hit-rate on Zipfian workloads, where recency alone captures access patterns. Partitioning-based variants (Hot/cold) showed modest hit-rate gains (+0.09–0.39%) but at 2–3× latency cost or requiring large hot-partition buffers (60%+ of capacity) to remain viable.

- **Zero-bookkeeping policies (FIFO)**: Maintained ClassicLRU's hit-rate (0.6611) while eliminating per-hit movement logic, but victim selection for wide buckets required O(n) scans that erased the zero-cost assumption. FIFO's advantage assumes O(1) victim identification; when that breaks, policy regresses.

- **Probe-aware policies (Robin Hood)**: Rebalancing probe distances to reduce variance achieved +8.6% hit-rate but doubled get/put latencies, resulting in Pareto domination by latency cost. The hit-rate improvement is unaffordable.

- **Segment and epoch-based policies**: Attempted to defer bookkeeping cost from hit path to eviction path, but deferred cost does not disappear—it compounds. Epoch tracking on every operation, segment metadata maintenance, and per-operation decay loops all incur cumulative overhead that exceeds ClassicLRU's simple vector operations. Structural indirection is not a solution.
