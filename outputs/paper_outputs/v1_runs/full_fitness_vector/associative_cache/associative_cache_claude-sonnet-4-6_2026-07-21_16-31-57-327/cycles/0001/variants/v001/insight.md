# Variant v001 Insight

## Helped

- hit-rate: Replacing position-order eviction with frequency-count eviction shielded high-reuse items from cold-burst displacement on the LIRS workload-a trace, where clusters of distinct one-time accesses repeatedly flushed the short LRU queue in the baseline.

## Hurt

- per-op-time: Both `get` and `put` now require a full O(n) linear scan over all items in the bucket to find the minimum or maximum `lfu_counter`, replacing the baseline's O(1) head/tail index access; the cost grows with bucket occupancy and shows up as elevated p50 and p99 latency on both operations.

## Takeaway

Switching `ClassicLRU` to LFU semantics — using the already-allocated `lfu_counter` field — meaningfully raises hit rate on a scan-heavy trace at zero metadata overhead, but the linear scan for the eviction and promotion candidates makes every cache operation proportionally slower. The trade-off is clear: LFU semantics are cheaper in miss cost but more expensive in per-op CPU cost; future work should explore whether a constant-time LFU structure (e.g., a sorted counter list) can recover the latency without surrendering the hit-rate gain.
