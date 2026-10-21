# Variant v003 Insight

## Helped

(None measured. `metadata-size` was the dimension this change was meant to improve. `per-op-time` was supposed to hold steady or improve. Both are missing from v003's fitness vector, so neither gain is confirmed.)

## Hurt

(None.) `hit-rate` stayed the same as the parent's. That is what the hypothesis predicted: ClassicLRU orders items only by their position in the bucket and never reads the per-item timestamp. So replacing the 16-byte `Instant` with a `u32` logical `stamp`, and the `usize` `lfu_counter` with a saturating `u32`, cannot change which item gets evicted on the eval path.

## Takeaway

On the eval path, the item's timestamp field was dead weight: removing wall-clock time from `Item` and moving TimestampLRU to a global `AtomicU32` logical clock leaves ClassicLRU's hit rate exactly unchanged. The point of the change (shrinking `Item<u64,u64>` from 40 to 24 bytes, with no `Instant::now()` call on every put and smaller elements to move on each ClassicLRU shift) is still unproven. The evaluation reported no `metadata-size` or `per-op-time` for v003, so those dimensions need to be re-collected before this variant can be ranked against the parent. One side effect to watch: the shared atomic clock is a contended write on every TimestampLRU hit and every insert, which may matter for multi-threaded throughput even though ClassicLRU only pays for it on `Item::new`.
