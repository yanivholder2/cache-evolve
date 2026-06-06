# Variant v003 Insight

## Helped

- per-op-time: Reducing `lfu_counter` from `usize` to `u16` compacts the Item struct, fitting more items per cache line and dramatically accelerating hot-path `get` operations (43% p50, 45% p99 faster).
- per-op-time: `put` operations also benefit from improved cache locality, though modestly (14% p50 faster), as the denser packing reduces memory pressure during bucket traversal.

## Hurt

(none)

## Takeaway

Shrinking the LFU counter to u16 delivered substantial latency wins—especially for reads—by improving CPU cache utilization, despite reporting the same aggregate metadata-size. The u16 ceiling is more than sufficient for LFU counters in the small bucket sizes this cache operates with, making this a clear win for access performance without sacrificing eviction policy behavior.
