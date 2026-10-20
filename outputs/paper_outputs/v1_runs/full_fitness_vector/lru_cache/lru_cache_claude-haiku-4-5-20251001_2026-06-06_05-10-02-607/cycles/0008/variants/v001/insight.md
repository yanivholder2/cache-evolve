# Variant v001 Insight

## Hurt

- **per-op-time (get p50)**: Median latency regressed 35% (34→46 ns) because deferred-promotion queue batching infrastructure (Vec allocations, threshold checks, batch flushes) adds constant overhead to every get operation. The workload remains in simple mode, so batching complexity is wasted overhead.
- **per-op-time (get p99)**: Tail latency regressed 75% (97→170 ns) from the same mechanism—when the main batch reaches swap threshold (4 items), two batch flushes trigger immediately, serializing four expensive frequency bucket updates at once instead of spreading them across operations.
- **per-op-time (put p50)**: Median put latency regressed 22% (72→88 ns) because put operations now check both batching conditions and mode-switch logic; even when inserting in simple mode, the infrastructure cost applies.
- **metadata-size**: LruCache struct size increased 33% (192→256 bytes) by adding `deferred_batch` and `swap_batch` Vec fields (64 bytes total) to maintain queue state. These remain mostly empty during simple-mode operation, where the workload spends all its time.

## Helped

- **per-op-time (put p99)**: Tail latency remained essentially flat (177→178 ns), suggesting the mode-switching logic at least didn't destabilize put worst-case latency.

## Takeaway

Deferred-promotion queue batching is architecturally mismatched to adaptive bucketing on underutilized caches. The hypothesis assumed the workload would trigger high-saturation mode frequently enough to amortize batching overhead, but this workload stays in simple mode (bounded-window eviction) because it never saturates. The batching infrastructure—which includes Vec allocations, boundary checks, and flush logic—imposes constant per-operation cost even when completely idle. Reducing batch size (16→8) and swap frequency (full→50%) were not aggressive enough to overcome the fixed overhead of maintaining two separate queues. The original cycle/0003/v003 baseline avoids this cost entirely by deferring no work; the delta teaches that deferred batching only pays off when the expensive path (frequency bucket updates) is the hot path—not when the fast path (simple mode) dominates.
