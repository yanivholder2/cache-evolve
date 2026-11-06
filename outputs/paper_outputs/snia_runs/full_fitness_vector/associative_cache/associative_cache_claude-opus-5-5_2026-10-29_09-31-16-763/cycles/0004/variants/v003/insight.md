# Variant v003 Insight

## Helped

- metadata-size: The `usize` `lfu_counter` became a `u32`, and the `Instant` timestamp became a `u32` tick from a logical clock. Placed after `key`/`value`, the two fields pack into one 8-byte word, so `Item<u64,u64>` reaches the 24-byte target the hypothesis set. This is the first time the dimension has moved. `AssociativeCache<u64,u64>` was untouched, as expected, since only the item layout changed.
- hit-rate: Not a gain, but the invariant held. Every cell matches the parent exactly. ClassicLRU (with LIP quarter-depth, BIP period 32 and the reverse scan kept as they were) never reads `timestamp` or `lfu_counter`, so the narrowing could not change any policy decision.

## Hurt

- per-op-time: Get and put both got slower at p50 and p99, the reverse of the predicted gain from denser bucket scans. The width change itself should only cut the bytes touched by `rposition`, by the ClassicLRU `remove`/`push` on a hit, and by the quarter-depth `Vec::insert` memmove. So the slowdown most likely comes from the new `item::tick()`. It is a global `AtomicU32::fetch_add` on every `Item::new`, which means a serializing locked read-modify-write on one shared cache line for every put. That cost can also leak into nearby timed gets if the benchmark interleaves them, and it gets worse if the benchmark runs on more than one thread. Get p99 and put p99 rose the most, which fits that cause. Run-to-run noise can't be ruled out, but the regression is consistent across all four measurements.

## Takeaway

Shrinking `Item` to 24 bytes is safe for hit rate and does fix metadata-size. But replacing `Instant::now()` with a process-wide atomic logical clock hid any cache-line benefit behind a new synchronization cost on the hot put path. The item-size change and the timestamp source should be judged separately. A next attempt should keep the 24-byte layout but fill `timestamp` without a shared atomic: a per-bucket counter like `bip_tick`, or no write at all when the policy never reads it. That would show whether the denser layout actually speeds up the scan.
