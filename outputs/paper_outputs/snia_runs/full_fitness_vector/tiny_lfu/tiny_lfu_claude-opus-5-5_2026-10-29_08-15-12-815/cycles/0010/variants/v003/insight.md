# Variant v003 Insight

## Helped

- per-op-time (get p50/p99): The hypothesis predicted this and it held. `get` now runs `IndexTable::find` instead of `key.clone()` plus building a `HashedKey` plus a hashbrown SIMD group probe over 24-byte buckets. `find` reads one 8-byte packed slot and then the `Node` line that the hit path needs anyway to return the value. Both the median and the tail dropped, so the map probe was a real part of get latency, and the sketch update is not the only remaining cost.
- per-op-time (put p50): The common put paths also gained. The update-path lookup uses the same tag-checked `find`, and the fill path's `insert` is a short probe to the first `EMPTY` at load ≤ 0.5, with no `HashedKey` construction and no chance of growth or rehash.
- hit-rate: No change in any cell, as the hypothesis required. Admission, eviction and `key_hash`/sketch indexing were not touched. The new table changes only how a key is found, not which keys are resident. The match confirms the backward-shift deletion keeps the index correct.

## Hurt

- per-op-time (put p99): The put tail got worse while every other percentile improved. The tail comes from the full-cache eviction path, which now runs `index.remove` with backward-shift deletion. That walks the whole rest of the cluster after the hole and may rewrite several slots, then a separate `insert` probe follows. hashbrown's erase only marks a control byte. Most clusters are short at load ≤ 0.5, but the occasional long one makes a full eviction pay for two cluster walks, and that rare cost is what p99 measures.
- metadata-size (TinyLfuCache struct): The cache header grew slightly. `IndexTable` keeps its own `mask: u32` and `len: usize` next to the `Vec`, while the `HashMap` with a zero-sized `PrehashedBuildHasher` kept its length inside its control structure. The 24-byte `Node`, which is the summary dimension, did not change.

## Takeaway

A packed `(tag | idx)` open-addressing index that leans on the key and hash already stored in `Node` takes a large share out of get p50/p99, so the hashbrown probe and the key clone were the main avoidable get costs left. The trade is that backward-shift deletion moves cost to the eviction path and raises put p99. The next index tweak should cut the cost of the eviction path while keeping the table. One option is to merge the `remove(old_h)` and `insert(key_h)` probes. Another is a lower load factor, which shortens the clusters that backward-shift has to walk.
