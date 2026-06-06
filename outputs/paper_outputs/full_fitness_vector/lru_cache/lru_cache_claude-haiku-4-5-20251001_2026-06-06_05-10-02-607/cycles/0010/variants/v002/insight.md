# Variant v002 Insight

## Helped

- **per-op-time (get p99)**: Quantized frequency levels reduce the number of bucket relocations triggered by access patterns, lowering tail latency in read operations by 39% (114 ns → 70 ns) compared to per-access frequency increments.
- **per-op-time (put p99)**: Batching accesses into discrete promotion thresholds (counter > 1 to promote level 1→2, counter > 12 to promote 2→3) reduces the frequency of bucket manipulation during flush operations, improving tail latency by 26% (297 ns → 220 ns).

## Hurt

- **hit-rate**: Coarse-grained quantization (4 levels instead of unbounded frequency) loses selectivity in distinguishing hot from moderately-hot items, reducing hit rate by 1.85% (0.6904 → 0.6719). Promotion thresholds are not sufficiently adaptive to the actual access patterns in the workload.
- **per-op-time (get p50)**: Quantized promotions with local counters introduce slightly more per-access bookkeeping than simple frequency increments, increasing typical get latency by 31% (35 ns → 46 ns).
- **per-op-time (put p50)**: Threshold-based promotion logic in `should_promote()` adds branch complexity during flush, increasing typical put latency by 15% (88 ns → 101 ns).

## Takeaway

Quantizing frequency into 4 discrete levels successfully reduces promotion churn and improves tail latencies (get/put p99), but at the cost of hit rate and median latency. The fixed thresholds (counter > 1 and > 12) are not flexible enough to track the actual access distribution in the workload, causing the cache to evict items that would remain hot under an unbounded frequency model. This variant trades selectivity for latency predictability — it works best for workloads where latency tail uniformity matters more than cache efficiency, but sacrifices overall hit rate compared to the parent's per-access frequency tracking.
