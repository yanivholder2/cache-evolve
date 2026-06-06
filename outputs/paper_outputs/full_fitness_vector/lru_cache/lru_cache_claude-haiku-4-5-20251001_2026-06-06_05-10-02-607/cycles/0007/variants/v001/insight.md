# Variant v001 Insight

## Helped

- **per-op-time (get p99)**: The two-batch swap strategy and fixed-size batch flushing reduce the total overhead of deferral by processing promotions in 16-item chunks, allowing CPU cache reuse and preventing the all-at-once pointer-chasing that spiked p99 to 114 ns in v002 (now 66 ns, a 42% improvement).
- **per-op-time (put p99)**: Swapping batches before processing and chunking the flush decouples new gets from being added to the queue being drained, allowing put() to handle queued work in predictable small batches rather than a potentially large accumulated queue (297 ns → 149 ns, a 50% reduction).
- **per-op-time (put p50)**: Chunked processing reduces median put latency by spreading flush work more evenly rather than serializing all accumulated promotions in one operation (88 ns → 72 ns).

## Hurt

- **per-op-time (get p50)**: The additional queue management and batch swapping add a small overhead to every get(), even those following a recent put that already started flushing; the two-batch design trades get p50 consistency for p99 tail latency reduction (35 ns → 42 ns, a 20% increase).
- **metadata-size**: The second batch vector (`current_batch`) adds 24 bytes to the LruCache struct overhead, increasing total per-cache memory by 1.5% (144 → 168 bytes).

## Takeaway

This variant validates the hypothesis that the put-side cost of complete deferral is tunable through algorithmic optimization of the queue-flush mechanism. By decoupling pending promotions (arriving during flush) from the currently-flushing batch, and processing the flushed batch in fixed-size chunks, the variant dramatically improves both get and put tail latency simultaneously—get p99 from 114 to 66 ns, and put p99 from 297 to 149 ns. The modest increase in get p50 latency reflects the cost of managing two batches on every get, a small price for eliminating the batching overhead spikes that dominated p99 behavior in v002. The deferred-promotion design now offers genuinely balanced latency across both read and write paths rather than trading read gains for write predictability.
