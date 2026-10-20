# Variant v001 Insight

## Helped

- per-op-time (get p50): Replacing SipHash with the in-file Fx multiply-rotate hasher makes each `map.get` cheaper. The gain is small, though, and well short of the hypothesis's prediction. Now that hashing is cheap, the lookup is no longer the main cost of a hit. A hit still touches `map`, `links[idx]` (for `set_visited`) and `nodes[idx]`, three separate memory locations.
- per-op-time (put p99): The tail improves most. This fits the two changes that only matter on a miss: the evict-and-insert path hashes twice (`map.remove` plus `map.insert`) and so benefits twice from Fx, and the `mem::replace` swap drops one `K::clone` of the evicted key.
- metadata-size (`LruCache<u64,u64>`): `BuildHasherDefault<FxHasher>` is zero-sized, while the default `RandomState` stores two `u64` SipHash keys inside the map. So the hasher swap alone cuts the fixed per-cache struct size and wins back more than v003's extra `links` `Vec` header cost.

## Hurt

- per-op-time (get p99): This was flat to slightly worse, so cheaper hashing does nothing for the slow tail of hits. That tail is set by cache misses on the separate `links` and `nodes` lines, not by hash computation.

## Takeaway

The hashing lever and the side-array layout lever do not stack additively on the hit path. Fx hashing removed only a small part of v003's get latency, and get p50 stayed above the 30 ns threshold the hypothesis set for "the side array costs real latency". So the second memory touch per hit (`links[idx]` and `nodes[idx]`) now dominates. Most of the hasher's value shows up on the miss path (put tail) and in the struct size. Narrowing map values to `u32` saves no memory for `u64` keys, because `(u64, u32)` buckets pad to 16 bytes just like `(u64, usize)`. Further get-latency work should target co-locating the visited bit with the data a hit already reads, not the hash function.
