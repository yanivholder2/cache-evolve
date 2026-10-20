# Variant v003 Insight

## Helped

- metadata-size: The value moved out of the arena and into the map bucket (`HashMap<K, Slot<V>>`), so `Node<u64,u64>` now holds only the key, which halves the per-node footprint as the hypothesis predicted. The bytes did not go away, though. They moved into the hash table, which this dimension does not measure, and every bucket grows from key+`u32` to key+`Slot { u32, u64 }`.

## Hurt

- per-op-time (`get`): The hit path now skips `nodes[idx]`, but it got slower instead of faster. At capacity 2048 the old 16-byte-per-node arena was small enough to stay mostly cache-resident, so the "second scattered line" the rationale wanted to remove was rarely a real miss. Meanwhile the wider buckets make the hashbrown table about 50% larger, and a 24-byte stride lets some buckets straddle two cache lines, so the probe that every hit pays for got more expensive. The `get` tail worsened along with the median, which fits a cost added to every probe rather than occasional noise.
- per-op-time (`put`): The rationale expected the miss path to hold, because `remove(&old)` and `insert` were already happening. Instead both now move wider buckets inside a larger table, and the existing-key update has to go through `get_mut` on that same bigger table. That made `put` slower at both the median and the tail.

`hit-rate` did not move. The change only altered data layout: eviction order, the ghost, the VISITED bitset and `find_victim` are identical, as intended.

## Takeaway

Putting values inline in hashbrown buckets to save one arena read backfires at this capacity. The arena read was cheap because the arena stayed resident, while widening the buckets from 16 to 24 bytes made every probe and every insert/remove pay more. This meets the hypothesis's own failure condition (`get` p50 did not drop), so the hit path's real floor is the cost of the map probe, not the value line. Future latency work should shrink or speed up the map bucket (for example, keep the `u32` index payload), not move more data into it. The smaller `Node` is not worth this trade.
