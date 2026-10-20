# Variant v003 Insight

## Helped

- **per-op-time (get p99)**: Lowering the saturation threshold to 45% causes the deferred-promotion condition to fire frequently on the test workload. This allows the vast majority of get operations to skip the pointer-chasing cost of immediate promotion, reducing tail latency by 36% (114 ns → 73 ns).
- **per-op-time (put p50)**: Conditional flushing logic (only flushing when necessary: queue depth > 32, leaving high saturation, or about to evict) slightly reduces the median cost of put operations by avoiding unnecessary flushes on every put, improving by 5% (88 ns → 84 ns).

## Hurt

- **per-op-time (get p50)**: The saturation check on every get (`map.len() / capacity`) incurs a division operation and branch even in low-saturation mode where promotions are still immediate. This uniform overhead degrades median get latency by 31% (35 ns → 46 ns), outweighing any benefit from having a stable check in place.
- **per-op-time (put p99)**: The dual-path promotion logic (conditional immediate vs. deferred in put) and queue-depth checks add latency branches to the put fast path, increasing tail latency by 7% (297 ns → 319 ns).
- **metadata-size**: The new `saturation_threshold` (f64) and `queue_depth_threshold` (usize) fields increase LruCache struct overhead by 16 bytes (144 → 160 bytes), adding memory cost without reducing memory per cache entry.

## Takeaway

This variant successfully preserves correctness (hit rate stable at 0.6904) and achieves the hypothesis goal: the 45% saturation threshold fires frequently enough on the test workload to deliver meaningful get p99 improvement (36% tail-latency gain via deferred promotions). However, the implementation reveals a critical trade-off: the uniform cost of computing saturation on every get (even when not deferring) causes measurable get p50 regression (31%), converting the deferred-promotion win into a distribution-wide performance penalty. The variant teaches that saturation checks must have negligible cost to be viable in hot paths, or the condition must be tracked asynchronously to avoid per-operation overhead.
