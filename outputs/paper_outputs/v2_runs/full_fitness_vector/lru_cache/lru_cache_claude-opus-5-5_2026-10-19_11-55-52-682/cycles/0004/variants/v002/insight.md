# Variant v002 Insight

## Helped

- per-op-time: The hit path in `get` is one map probe plus a bit write, so swapping SipHash-1-3 for a single rotate-xor-multiply round on the `u64` key cut directly into the dominant cost. This is the first per-op-time movement since node compaction stopped helping, and it confirms that hashing, not node layout, was the bottleneck. `put` gained about the same at p50 and more at p99. A miss-path `put` hashes the key up to three times (probe, `remove` of the old key, `insert` of the new one), so every probe got cheaper. The `evicted_key.clone()` that was dropped is close to free for `u64` keys, so most of the tail gain probably comes from cheaper hashing rather than the removed clone.
- metadata-size: The summary (per-`Node` bytes) did not move, as expected. The `LruCache` struct itself shrank because `BuildHasherDefault<FxHasher>` is zero-sized, while `RandomState` stores two `u64` SipHash keys inline. That is a one-time saving per cache instance, not per entry. The narrower `u32` map value probably saves nothing per bucket either, since a `(u64, u32)` entry pads back to 16 bytes.

## Hurt

No dimension regressed. hit-rate stayed the same, as the hypothesis required: only the map's hasher and value width changed, and neither the queue nor the hand logic was touched, so eviction order is bit-for-bit the same.

## Takeaway

On a relink-free SIEVE, per-op latency is now almost entirely the cost of the hash-map probe. That is why replacing SipHash with an Fx-style hasher moved `get` p50 when two cycles of node-layout work did not. The gain on `put` shows the same lever applies to every probe on the miss path. The next latency levers are therefore cutting the number of probes per `put` (for example, merging the miss-path lookup and insert with an entry-style API) or probe locality. Node shape is no longer worth pursuing for speed. The trade-off is that the map is no longer resistant to hash-flooding (DoS), which no fitness dimension measures.
