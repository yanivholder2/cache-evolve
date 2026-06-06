# Variant v001 Insight

## Helped

- **per-op-time (get p99)**: Tail latency improved from 147 ns to 140 ns by deferring expensive bucket reorganization operations from the get() fast path to batch flush points, eliminating pointer-chasing overhead on every access.
- **hit-rate**: Frequency-aware eviction logic preserved; deferred updates are flushed before eviction decisions, ensuring accurate frequency signals for LRU selection without compromising hit rate (maintained at 0.6904).

## Hurt

- **per-op-time (put p99)**: Tail latency degraded severely from 190 ns to 311 ns because pending frequency updates (accumulated over up to 64 accesses) are flushed in a batch before eviction, creating burst overhead when the arena is full and multiple deferred bucket reorganizations execute synchronously.
- **metadata-size**: LruCache struct grew from 120 to 152 bytes due to two new fields: `pending_freq_updates` queue and `access_counter` for tracking periodic flush intervals.
- **per-op-time (put p50)**: Median unchanged, but introduces unpredictable variance; batch flushing at eviction boundaries concentrates work into single operations.

## Takeaway

Lazy-batch deferral successfully reduces get-path latency by amortizing bucket reorganization work, but the strategy transfers that cost to put operations in the form of eviction-time flushing. When the cache fills and needs to evict, all accumulated pending frequency updates are processed at once, creating a p99 spike (64% slower). The design trades smooth, predictable put performance for improved get latency and maintains correctness. For workloads dominated by reads, this is a win; for write-heavy or eviction-heavy workloads, the concentrated batch work at eviction boundaries becomes a liability.
