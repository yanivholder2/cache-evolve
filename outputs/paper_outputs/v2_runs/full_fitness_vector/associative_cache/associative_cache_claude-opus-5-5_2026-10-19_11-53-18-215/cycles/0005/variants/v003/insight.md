# Variant v003 Insight

## Helped

- metadata-size: `Item<K, V>` is now just `{ key, value }`. Dropping the `u32` `timestamp` and `u32` `lfu_counter` removes 8 bytes per item, so `Item<u64,u64>` reaches its floor, with no extra per-item bytes beyond the payload. The `AssociativeCache<u64,u64>` handle is unchanged. The dimension measures only `Item` and the handle, so the improvement comes entirely from `Item`.
- per-op-time (get): `hit()` on the SLRU path is purely positional: it rotates and shifts `Item`s within the bucket. With a 16-byte stride instead of 24, every scan and rotate moves a third fewer bytes and more items fit per cache line. Get latency improved slightly at the median and clearly at the tail, where the longest moves happen.
- per-op-time (put): `Item::new` no longer calls `tick()`, so every put skips a thread-local clock read-modify-write. The parent's `mem::replace` + `rotate_left` miss path and the `remove_at`/`insert` ghost-hit path also move narrower elements. The put tail benefits most because its slowest branches shift the most items.

## Hurt

## Takeaway

Under the merged-bucket SLRU, `timestamp` and `lfu_counter` were pure dead weight. Removing them left hit rate exactly unchanged, which confirms that the evaluated path never reads them. The smaller memory footprint and the latency gains both come from the same source: a narrower element stride in a design whose cost is dominated by in-bucket memmoves. The latency gain shows up mostly in the p99 tails, because the tails are where the longest shifts happen. The LFU side array that the fallback policies still need lives in the bucket and is used only when SLRU is off, so neither the evaluated path nor the metadata-size dimension sees it.
