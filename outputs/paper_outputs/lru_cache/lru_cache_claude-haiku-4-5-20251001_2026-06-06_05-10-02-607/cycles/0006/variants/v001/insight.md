# Variant v001 Insight

## Helped

- **per-op-time (put p50)**: Median put latency improved 18% (72→59 ns) by deferring frequency bucket updates to a promotion queue, spreading unlink/relink work across multiple operations instead of paying it all in one put() call. This amortization reduces the typical latency spike on insertions.

## Hurt

- **per-op-time (get p99)**: Tail latency regressed 34% (97→130 ns) because queuing promotions introduces key cloning overhead in `queue_promotion()`, and when queue depth exceeds the threshold (8 items), the drain operation runs on the critical get() path, causing unlink/relink operations to become part of get's worst case. For a mostly underutilized workload that never builds a deep queue, this draining overhead is wasted.
- **per-op-time (put p99)**: Tail latency regressed 21% (177→214 ns) because put() now flushes the entire pending promotion queue before eviction to maintain accurate frequency state; the queue accumulation, though rare, becomes expensive when it does flush.
- **metadata-size**: LruCache struct size increased 25% (192→240 bytes) due to queue infrastructure: `pending_promotions` vector (24 bytes), `queue_threshold` and `drain_batch_size` counters (16 bytes), and `saturation_low_threshold` for mode-transition hysteresis (8 bytes).

## Takeaway

Incremental queue draining reduces median put latency by distributing work, but the strategy introduces per-operation overhead (key cloning, validation checks, batched draining) that penalizes tail latency for gets. For this workload—which remains mostly underutilized and never triggers high-saturation mode—the queue infrastructure sits mostly idle, making its overhead unprofitable. The design succeeds at smoothing put() median cost but fails the overall latency goal because it shifts the worst case from puts to gets. Incremental batching may be viable for high-frequency workloads where queues consistently build and drain, but on light workloads, immediate promotion is preferable to the overhead of deferral.
