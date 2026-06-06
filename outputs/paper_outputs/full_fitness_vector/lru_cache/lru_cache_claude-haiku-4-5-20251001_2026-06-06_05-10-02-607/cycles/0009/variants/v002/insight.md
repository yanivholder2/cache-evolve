# Variant v002 Insight

## Helped

- **metadata-size**: Replacing frequency buckets and min_freq tracking with a single global operation_counter reduced the LRU structure from 120 to 96 bytes; the simpler state does not require maintaining bucket pointers across the heap.
- **per-op-time (get p99)**: Tail latency improved from 147 ns to 69 ns by eliminating pointer-chasing overhead in bucket unlinking/relinking during frequency updates on cache hits.

## Hurt

- **hit-rate**: Dropped from 0.6904 to 0.6652 because time-windowed recency (evicting least-recently-accessed) is weaker than frequency-based eviction; recency alone cannot distinguish between items accessed once recently versus items accessed many times.
- **per-op-time (put p99)**: Catastrophic regression from 190 ns to 600 ns; the linear scan of all nodes in `find_evictable_node()` becomes O(N) without bucket indexing, creating massive tail latency on puts under load.
- **per-op-time (get p50)**: Slight increase from 35 ns to 45 ns; the decayed operation counter increments and `last_access_op` updates add minor overhead even on hits.
- **per-op-time (put p50)**: Slight increase from 70 ns to 75 ns due to decay triggering logic during eviction.

## Takeaway

Replacing frequency buckets with time-windowed recency successfully reduced code complexity and improved get tail latency, but failed because frequency information itself is essential for selectivity: the parent variant's O(1) bucket indexing was critical for avoiding O(N) eviction scans on puts. Time-windowed LRU cannot capture cumulative importance (frequency), only recency, resulting in 3.6% hit-rate loss and a 215% explosion in put p99 latency. This variant demonstrates that simpler eviction logic (linear scan) cannot substitute for the selectivity that frequency tracking provides without reimposing the cost through catastrophic tail latency.
