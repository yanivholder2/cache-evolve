# Variant v003 Insight

## Helped

- **per-op-time (put p99)**: Batching frequency promotions and draining them deterministically during put() reduced tail latency from 190 ns to 163 ns by concentrating bucket-structure updates into fewer, larger batch operations rather than scattering them across individual get() calls.

## Hurt

- **per-op-time (get p50)**: Each get() now incurs a vector push and conditional drain check, increasing typical latency from 35 ns to 45 ns due to buffer-management overhead on the hot path.
- **per-op-time (get p99)**: Tail latency increased from 147 ns to 187 ns because accumulated deferred promotions in the buffer can create larger batch work when the threshold is reached, and the vector push operation itself adds measurable cost to the most frequent operation.
- **per-op-time (put p50)**: Typical put latency increased from 70 ns to 86 ns because every put() now calls drain_promotion_buffer() unconditionally, adding overhead before the main insertion logic.
- **metadata-size**: LruCache structure grows by 24 bytes (120 → 144) to store the promotion buffer Vec<usize> allocation header.

## Takeaway

Opportunistic batch buffering of frequency promotions improves put p99 tail latency by amortizing bucket operations, but this comes at a steep cost to get latency (the hot path). The vector push and conditional check on every get() become the new bottleneck, outweighing the benefit of deferred updates by 27 ns of put improvement versus 40 ns of get-p99 regression. The hypothesis assumed that avoiding per-operation conditionals would keep hot-path overhead minimal, but the buffer-management cost itself (append + branch) proved more expensive than the original immediate bucket operations. This demonstrates that aggressive batching on the critical path can paradoxically increase latency if the batching primitive itself carries non-negligible overhead.
