# Variant v003 Insight

## Helped

- per-op-time: Reducing sketch depth from 4 to 2 halves the loop iterations in `increment()` and `estimate()`, delivering ~33% faster get operations and ~26% faster put operations due to reduced sketch traversal overhead.
- metadata-size: Halving the depth from 4 to 2 cuts the sketch heap from 65,536 to 32,768 bytes (50% reduction) since heap usage scales linearly with depth.

## Hurt

- hit-rate: Shallowing the sketch to 2 rows increases hash collision probability and degrades frequency estimation accuracy, causing the cache to misidentify cold vs. hot items in eviction decisions, reducing hit-rate by 1.6 percentage points (0.7362 → 0.7243).

## Takeaway

Reducing sketch depth to its minimal viable level (2 rows) creates a severe accuracy-speed tradeoff. The 25–33% latency win comes at the cost of 1.6% hit-rate loss, revealing that frequency estimation precision is critical to LFU eviction quality. The extremely shallow geometry (2×3) sacrifices too much discrimination power for this workload; the interaction between reduced depth and the LRU-only victim selection amplifies the impact of poor estimates. This suggests a practical lower bound exists around depth 3, where geometric austerity reaches the point of diminishing returns.
