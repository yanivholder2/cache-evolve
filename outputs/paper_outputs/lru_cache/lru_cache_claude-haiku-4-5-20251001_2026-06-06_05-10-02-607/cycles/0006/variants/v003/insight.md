# Variant v003 Insight

## Helped

- per-op-time (get p50): Removing per-operation frequency counter increments from the hot path reduced common-case get() latency by eliminating atomic operations and counter updates.
- per-op-time (get p99): Simplified get() code path eliminates frequency-tracking branch logic, improving tail latency predictability.
- per-op-time (put p50): Replacing frequency tracking with a single global operation counter increment reduces put() common-case overhead; most puts complete without hitting the eviction path.

## Hurt

- metadata-size (Node): Added `last_access_count: u64` field increases per-node overhead by 8 bytes, directly tied to the operation-count-based tracking mechanism.
- metadata-size (LruCache): Added `operation_counter: u64` field to track global operation sequence increases cache struct overhead by 8 bytes.
- per-op-time (put p99): The bounded-window eviction candidate search introduces a new worst-case latency spike; scanning the window to find the minimum `last_access_count` can cause cache misses or unexpected pointer chasing when the window contains cold nodes.

## Takeaway

Operation-count-based recency successfully reduces the common-case latency of both get() and put() by eliminating the per-operation frequency counter overhead. However, it trades this improvement for: (1) increased metadata footprint (8 bytes per node), and (2) a severe p99 regression in put() due to the eviction candidate search. Hit rate remains unchanged at 0.6652, indicating that operation-count-based recency does not provide better selectivity than FIFO on this workload—suggesting the bounded window is either too small, or this trace lacks sufficient frequency skew to benefit from recency-aware eviction. The variant demonstrates that frequency tracking's latency cost is real, but replacing it with bounded-window recency introduces a new performance cliff at high quantiles.
