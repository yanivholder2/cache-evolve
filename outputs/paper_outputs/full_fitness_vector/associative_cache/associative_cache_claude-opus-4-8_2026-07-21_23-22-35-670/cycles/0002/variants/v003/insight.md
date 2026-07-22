# Variant v003 Insight

## Helped

- metadata-size: Removing the `lfu_counter: usize` field (and its `Item::new` initializer, `Display`, and tests) drops `Item<u64,u64>` from a `key`+`value`+`lfu_counter` layout to a packed `key`+`value` (8+8) layout, cutting one full word off every item.

## Hurt

- per-op-time: The p99 tails on both `get` and `put` drifted upward while p50 (the summary) held flat; with the removed field never read on the `ClassicLRU` hot path, this is tail-latency noise rather than a cost of the field cut.

## Takeaway

`lfu_counter` was alive only for the off-path `Lfu` policy, so on the pure-`ClassicLRU` eval path it was as removable as `timestamp` was — cutting it shrank `Item` metadata by a third with zero hit-rate change. Neutralizing `Lfu` to positional no-ops while keeping every `EvictionPolicyKind` ordinal intact preserved the JNI surface, confirming that dead-weight fields carried only by unused policies can be peeled off monotonically.
