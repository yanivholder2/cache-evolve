# Variant v002 Insight

## Helped

- per-op-time (`get` p99): The `get` tail came down while its median did not. This fits the hit path of the new `ResidentIndex::find`: it goes straight to the `(K, u32)` bucket and reads the key and value word with no separate control-byte load first. At a load factor of 0.5 or less, almost every hit is found in the home bucket or the next one, so the slow cases that remain are short.

## Hurt

- per-op-time (`get` p50, the summary): The median did not drop below the parent as the hypothesis predicted; it got slightly worse. A hit still costs one hash computation (here an extra Fibonacci multiply on top of FxHash), one bucket load and one load of `slots[i].value`. That is the same work hashbrown did with its SIMD group scan, so taking out the metadata byte does not save anything you can measure at p50.
- per-op-time (`put`, mainly p99): Two of `put`'s paths got more expensive. First, the hit check on `put` is usually a miss, and a linear-probe miss has to walk to the next `EMPTY` bucket, where hashbrown rejects most misses with one group compare. Second, every eviction calls `remove`, and backward-shift deletion re-hashes each occupied bucket that follows the hole and may move it. That cost grows with cluster length, which explains why the tail grew more than the median.
- metadata-size (`S3Fifo` header): The struct header grew by 16 bytes. `ResidentIndex` stores `Vec` + `mask` + `shift` + `len`, plus padding, which is wider than hashbrown's raw table header with a zero-sized hasher. The per-entry `Entry` is still 8 bytes, so the summary did not change.
- hit-rate: No change, as intended. The table is a drop-in key→packed-word map, it is called the same number of times from `get`, `put`, `packed_mut`, `alloc_with_key` and `release`, and the eviction decisions are unchanged.

## Takeaway

Swapping hashbrown for a hand-written linear-probe table at load 0.5 left `get` p50 flat. By the hypothesis's own test, that means the hash table internals are no longer what limits `get`. What remains is the fixed cost of the hash plus two dependent loads (the bucket, then `slots[i]`). The custom table also makes misses and deletions slower, and those costs land on `put`'s tail. Further `get` gains will probably require removing the second dependent load, for example by storing the value inline in the bucket or prefetching it, rather than tuning the probe again. Keeping hashbrown is the better trade unless that layout change comes with it.
