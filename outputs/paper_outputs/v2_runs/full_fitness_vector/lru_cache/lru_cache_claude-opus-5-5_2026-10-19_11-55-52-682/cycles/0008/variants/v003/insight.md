# Variant v003 Insight

## Helped

- per-op-time (`get` p50): A hit now does test-then-set on the VISITED bit in the `u32` map bucket that `get_mut` already pulled in, then reads `nodes[idx]`. The read-modify-write on the separate `visited` bitset is gone. The median hit got slightly faster and just clears H03's bar. That makes the bitset access the last avoidable memory touch on the hit path. The gain is small because the probe and the value line still dominate.
- per-op-time (`put` p50): On the common miss path, `map.insert(key, idx as u32)` stores the flag already cleared. This removes the `vbit_clear` that v001 added on every slot reuse and push, which takes back the small fixed cost v001's insight blamed for its `put` p50 regression.
- metadata-size (struct): `LruCache<u64,u64>` lost the `visited` `Vec` header, which reverses v001's struct growth. The node stays at 16 bytes because the flag now uses a spare bit in the map value, so there is no per-slot heap side-array for it either.
- hit-rate: Held exactly as predicted. The flag keeps the same lifecycle: clear on insert, set on hit, tested and cleared in the same order on small-head pops and in the hand walk. Eviction order is identical to v001.

## Hurt

- per-op-time (`put` p99): Each step of the SIEVE hand in `find_victim` now does an Fx hash plus a `map.get_mut` probe of `nodes[i].key` instead of a dense, L1-resident bitset access. A long sweep over visited main nodes now costs a scattered map probe and a bucket write per node, and the eviction tail regressed sharply. Small-head pops are slower too, because they hash the key twice (the map probe, then `hash_one` for the ghost). The rationale's optional step of hashing once was not applied.
- per-op-time (`get` p99): The tail rose slightly. The first hit on a cold key now dirties a line in the large, scattered map table instead of a compact bitset word. One plausible cause is that the hand sweep's map writes, which clear VISITED, keep re-dirtying buckets that later hits must set again.

## Takeaway

Storing VISITED in the map value's spare high bit puts the flag next to the map bucket the hit path already probes. It removes the third structure from both `get` and the insert path, so medians and struct size improve while eviction order stays the same. The cost is that every flag access off the hit path needs a key-to-bucket lookup. The SIEVE hand and small-queue promotion turn cheap index-addressed bit operations into hash-plus-probe operations, which costs much more at `put` p99 than the sub-nanosecond `get` p50 gain is worth. The flag sits in the right place for readers that come in by key, but the wrong place for the eviction scan, which reaches nodes by index. Any recovery has to give the hand index-addressed access again or make it touch fewer nodes.
