# Variant v002 Insight

## Helped

- per-op-time (get p99): Get p99 fell sharply. The new table uses backward-shift deletion with no tombstones, and the eviction path now removes the victim's key *before* inserting the new one. Probe chains therefore cannot fill with deleted-slot markers under constant churn. This is the strongest evidence so far for the parent insight's suspicion: the parent's get-p99 spike came from tombstones left by hashbrown's insert-then-remove order, not from noise.
- metadata-size (`LruCache<u64,u64>` header): The header shrank. `Box<[u64]>` (a 16-byte fat pointer), a `u32` mask and `RandomState` together are smaller than a `HashMap` handle. The summary node size is unchanged because `Node` was left byte-identical.
- hit-rate: This dimension did not move, so it is not a real gain. It shows the change was representation-only: `on_hit`, `unlink`, `push_back` and victim selection were kept intact, so eviction order is identical.

## Hurt

- per-op-time (get p50, the summary metric): Get p50 regressed, which disproves the hypothesis's central prediction. Every iteration of the hand-written scalar linear probe does a bounds-checked `slots[i]` load, a fragment compare and a `nodes[idx]` bounds check. hashbrown instead tests 16 control bytes per SIMD group. Fewer cache lines touched per hit did not make up for the extra per-probe instructions. The density premise was also off: `(2 * (capacity + 1)).next_power_of_two()` rounds 131,073 up to 524,288 slots. That gives a load factor near 0.25 and a 4 MB table, roughly the same footprint as hashbrown's buckets, not the halved 2 MB the rationale assumed.
- per-op-time (put p99): Put tail latency rose because the evicting put now costs much more. `remove_by_key` rehashes the victim and probes for it. `remove_slot` then recomputes a full SipHash `hash_one` for *every* entry it walks in the following cluster, to find that entry's home slot. Finally `first_empty` probes again. The parent's evicting put was one entry probe plus one `map.remove`. Put p50 barely moved, because most puts are hits or short clusters.

## Takeaway

A packed `u64` slot table with backward-shift deletion fixes the tail: removing the victim before inserting, with no tombstones, removed the get-p99 problem the parent hit. In safe Rust, though, it loses get p50 to hashbrown's SIMD group probing, so get p50 is not bound by cache lines per lookup. If this table is iterated on, two fixes come first. One is to size it as `(2 * capacity).next_power_of_two()`, so power-of-two capacities don't double the table. The other is to recover home slots during backward shift without a SipHash per entry, for example by storing the hash bits that give the home slot in the fragment. Separately, the tombstone-free removal order is worth carrying back to the HashMap parent on its own.
