# Variant v001 Insight

## Helped

- **hit-rate**: Doubling sketch width from 4 to 8 reduces hash collisions in the frequency estimation table, improving accuracy of per-key counts and leading to better eviction decisions (0.7334 → 0.7390).

## Hurt

- **per-op-time**: Wider sketch increases memory footprint and reduces cache locality; both `increment()` and `estimate()` now operate on twice as much data, raising p50 latency from 90 ns to 144 ns.
- **metadata-size**: Sketch heap doubles from 49152 to 98304 bytes as width increases (depth remains 3); proportional to the width expansion.

## Takeaway

Increasing sketch width to 8 successfully recovers the hit-rate loss that depth reduction incurred, validating the hypothesis that the two levers could synergize. However, the memory and latency penalty of doubling width outweighs the hit-rate gain in this workload: per-op time rises 60%, negating the speed advantage that depth-3 achieved in v002. The tradeoff suggests that for this cache size and workload, width expansion beyond factor 4 becomes counterproductive—the wider sketch's accuracy benefit does not justify the overhead.
