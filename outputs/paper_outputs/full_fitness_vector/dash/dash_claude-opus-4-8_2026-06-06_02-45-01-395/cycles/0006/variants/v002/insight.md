# Variant v002 Insight

## Helped

- per-op-time: Pre-reserving `target_heat` to the full normal-bucket capacity (`num_of_normal_buckets * bucket_size`) in `try_new` removed the mid-run `HashMap` rehash spikes — alloc + full reinsert — from the get path, pulling the get p99 tail down while p50 stayed flat.
- hit-rate: A capacity-only change touches no gate logic or `entry`/`remove` calls, so the parent's hit-rate carried over bit-for-bit as predicted.
- metadata-size: `with_capacity` resizes segment container state, not per-entry layout, so the 16-byte `Item<u64,u64>` floor held intact.

## Hurt

- per-op-time: The hypothesis expected both get and put p99 to fall, but only the get tail benefited — put p99 edged up rather than down, showing put's tail spikes do not originate from `target_heat` rehashing and stay bounded by something else on that path.

## Takeaway

Pre-sizing the side table is a real but one-sided latency win: eliminating rehash reallocations cuts the get p99 tail without costing hit-rate or metadata size, but it leaves put p99 untouched. The rehash cadence the cross-cycle insight blamed for "trading tail latency for size" is specifically a get-path cost here — the put tail is gated by a different mechanism that capacity reservation can't reach.
