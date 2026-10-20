# Variant v001 Insight

## Helped

- per-op-time (`get`, the summary): `get` now does one `index.get_mut(key)`. It reads and bumps the 2-bit freq in the same bucket word it already loaded, then does one bounds-checked load of `slots[i].value`. The parent did a hash lookup, then a separate read-modify-write on `freqs[i]`, then the slot load. Removing that extra array access and its bounds check undoes the problem the parent's insight identified, where a second memory stream was touched on every hit. The p99 moved the same way as the p50, which fits a lower per-hit memory cost rather than a noise change in the median only.
- per-op-time (`put`): `put` got faster too, even though the hypothesis expected it to pay one extra hash probe (`packed_mut`) for each main or small victim examined. The hit path only adds a `& SLOT_MASK`, and the miss path no longer writes `freqs` on alloc. These savings are small, so they cannot explain an improvement this large on their own, and the probe cost was expected to outweigh them. The parent's insight had already flagged its `put` regression as likely benchmark drift. So part of this rebound is probably that drift reversing, not a real benefit of the packing.
- metadata-size (`S3Fifo` header): Deleting the `freqs: Vec<u8>` field removes exactly one `Vec` header from the struct. Per slot, the separate 1-byte `freq` is also gone. It now uses the 2 spare high bits of the `u32` that `index` already stored, and the `(u64, u32)` bucket is still 16 bytes (now asserted in `entry_is_8_bytes`). Frequency metadata therefore costs nothing extra per slot.

## Hurt

- (none observed) hit-rate: Unchanged, as intended. Freq 0 is implicit in `alloc_with_key` inserting a bare `i`. `release` drops the freq along with the index entry. `evict_main` and `evict_small_inner` make the same `> 0` and `> 1` decisions as before, just read from the packed word. Capacity is now capped at 2^30 slots, which costs nothing at the evaluated size.

## Takeaway

Where the frequency counter lives decides `get` latency more than `size_of::<Entry>` does. Putting the 2-bit counter in the spare high bits of the hash-index value means a hit touches only the bucket it has already probed plus the value slab. That keeps the parent's 8-byte `Entry` and also wins back, and more than wins back, the `get` latency the parent lost with its separate `freqs` array. Paying an extra hash probe on the eviction scan is a good trade when `get` is the metric. However, the `put` gain should be re-measured side by side with the parent before it is credited to this change. I could not check the requested `h01_split_report` `settled_small` invariant from the eval artifacts. The unchanged hit-rate and the identical decision logic suggest it holds.
