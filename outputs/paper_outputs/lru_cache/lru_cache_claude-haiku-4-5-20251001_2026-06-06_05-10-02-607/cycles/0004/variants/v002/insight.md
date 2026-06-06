# Variant v002 Insight

## Helped

- **per-op-time (get p99)**: Deferring frequency promotions from the get fast path to put() eliminates pointer-chasing overhead in the hot read path, reducing tail latency by 23% (147 ns → 114 ns).

## Hurt

- **per-op-time (put p50)**: The flush of pending promotions at the start of put() adds bucket manipulation overhead to typical put operations, increasing median latency by 26% (70 ns → 88 ns).
- **per-op-time (put p99)**: In read-heavy workloads where many pending promotions accumulate before a put, the batch flush can spike tail latency by 56% (190 ns → 297 ns), making put operations less predictable.
- **metadata-size**: The `pending_promotions` queue adds 24 bytes to LruCache struct overhead (120 → 144 bytes).

## Takeaway

This variant successfully isolates get latency variance from frequency promotion work by deferring updates to put time. The 23% get p99 improvement (147→114 ns) comes at the cost of making put operations less predictable, especially in read-heavy workloads where promotions batch up. The frequency-aware eviction strategy remains effective (hit rate stable at 0.6904) because pending promotions are flushed before eviction decisions. This trade-off is suitable for read-dominated workloads where get tail latency matters more than put latency consistency, but unsuitable for write-heavy or balanced workloads where the put penalty outweighs the get gain.
