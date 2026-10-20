# Variant v003 Insight

## Helped

- metadata-size: Swapping the 16-byte `Instant` for a `u32` logical tick and narrowing `lfu_counter` to `u32` (placed next to each other) makes `Item<u64,u64>` exactly key + value + 8 bytes with no padding. This is the first time the dimension has moved, and it landed exactly where H03 predicted.
- per-op-time: Put improved sharply at both p50 and p99, and get's p99 tail also improved. On the put path, `Item::new` now calls a thread-local wrapping increment instead of an `Instant::now()` vDSO call. Each `Vec` shift during insert, demotion and promotion also moves 40% fewer bytes per item, so more of every paired bucket fits in each cache line.

## Hurt

- per-op-time: Get's p50, which is the dimension's summary, got slower than the parent, against the hypothesis. On a ClassicLRU hit the code still does a `tick()` plus an in-slice `rotate_left`, and both should be cheaper now. This overlay doesn't show where the extra median cost comes from. It could be TLS access codegen in the `CLOCK.with` closure or benchmark noise, and it needs a closer look before the "cheaper clock helps get" claim is accepted.

## Takeaway

Shrinking per-item metadata from wall-clock to logical 32-bit fields costs nothing in policy quality. Hit rate stayed effectively flat, which shows the wrap-safe tick ordering keeps v003's demotion and LRU behaviour intact. The gains go mainly to writes, where item creation and in-bucket shifting dominate, and not to the median read. So removing `Instant::now()` was not the cause of v003's get latency, as H03's rationale assumed. Keep the 24-byte `Item` as the new base, but look for the get-path cost somewhere else, such as the hit-side `rotate_left` or the per-hit clock write.
