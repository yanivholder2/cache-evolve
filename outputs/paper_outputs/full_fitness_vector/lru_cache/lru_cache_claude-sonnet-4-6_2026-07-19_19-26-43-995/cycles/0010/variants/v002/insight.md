# Variant v002 Insight

## Helped

- **hit-rate**: Frequency-biased eviction among the 4 oldest probationary candidates produced a negligible uptick — the change is effectively noise, not a genuine gain from the ordering heuristic.

## Hurt

- **per-op-time**: Every `get()` now calls `freq_bucket()` (a full `DefaultHasher` invocation plus a multiply-shift) and writes to `self.freq[bucket]`, pushing the p50 get latency from 0 ns to 100 ns — a full timer-tick regression on the hot read path.
- **metadata-size**: The `freq: [u8; 256]` field added 256 bytes to the `LruCache<u64,u64>` struct (208 → 464 bytes); `Node<u64,u64>` stays at 16 bytes so the tracked summary metric is unchanged, but per-instance overhead more than doubled.

## Takeaway

Scanning 4 oldest probationary candidates and evicting the lowest-frequency one does not improve hit-rate on workload-a because the probationary queue is intentionally tiny (capacity/8 ≈ 256 slots), giving the frequency counters too little surface area to separate warm from cold within any realistic 4-item window. Meanwhile, the unconditional `freq_bucket()` hash on every cache hit adds measurable latency overhead, making this change a net loss: the cost is real but the benefit is not.
