# Variant v001 Insight

## Helped

- per-op-time: Removing the process-wide `static LOGICAL_CLOCK` and its `AtomicU32::fetch_add` from `Item::new` dropped the locked read-modify-write on a shared cache line from every put. Timestamps now come from a plain `wrapping_add` on a `clock: u32` field inside the bucket, which is already in cache. Put improved the most at both p50 and p99, which fits the parent insight's diagnosis that the atomic sat on the put path. Get also improved at both percentiles even though the hit path now does an extra non-atomic store (`items[index].timestamp = next_tick()` in `get_from_position`). So a local store costs much less than the shared atomic it replaced. Get p50 also landed at or below the cycle 0003 level quoted in the hypothesis, so environment drift does not explain the cycle 0004 regression.
- hit-rate: Not a gain, but the invariant held. All four cells match the parent exactly. ClassicLRU's order depends on the LIP quarter-depth insert, BIP period 32 and position in the `Vec`, and all of those are unchanged. Timestamps are only compared within one bucket, so swapping a global clock for a per-bucket one keeps every relative ordering.
- metadata-size: Not a gain, but the invariant held. `Item<u64,u64>` keeps the field order `key, value, lfu_counter: u32, timestamp: u32` and stays 24 bytes. Adding one `u32` to `AssociativeCacheBucket` does not change `AssociativeCache<u64,u64>`, because buckets are allocated on the heap, outside that struct.

## Hurt

## Takeaway

The cycle 0004 per-op regression came from the global atomic logical clock, not from the 24-byte layout or the environment. With the atomic gone, the denser `Item` brings get and put back to the speed of the 40-byte baseline or better, at no cost to hit rate. A per-bucket, non-atomic counter (the same pattern as `bip_tick`) is the right way to fill per-item recency fields. This variant still stamps `timestamp` on every insert and hit, though. If the active policy never reads it, skipping those stores is a further micro-optimisation worth isolating.
