# Variant v001 Insight

## Helped

- hit-rate: In v003, a newcomer that lost the frequency contest was dropped right away. Here every newcomer enters a 1% LRU window, so it can build sketch counts before it has to face the probation victim. Hits in probation are promoted into an 80% protected segment that the admission contest never draws victims from. The gain is largest at the small cache size on both traces. Small caches are where rejecting bursty newcomers at the door and evicting reused items from a single LRU cost v003 the most.
- metadata-size: The per-node size went down, which is more than the hypothesis asked for (it only had to stay at 48 bytes). `prev` changed from `Option<usize>` to a raw `usize` with a `NIL` sentinel. The sentinel and the 2-bit region tag share the masked top bits, which removes the 8-byte `Option` discriminant word. The SLRU therefore costs no per-node space at all.
- per-op-time: `get` now hashes once instead of twice. The map is keyed by `HashedKey` with a pass-through `PrehashedHasher`, so the single `key_hash` serves both the sketch and the map probe. Removing that second hash, plus a smaller node that fits more densely in cache, accounts for the drop in median and p99 `get` latency.
- per-op-time (put): `put` also got faster. This is despite extra work on a full cache: every put now hashes both the window candidate and the probation victim, and always does a map remove and insert, where v003 could stop early after rejecting the newcomer. Reusing the precomputed hash for the map lookup and for the evictee's removal saves more than those extra steps cost.

## Hurt

- metadata-size (cache struct): The fixed `TinyLfuCache` header grew. The single head/tail pair was replaced by three `RegionList` records (head/tail/len), plus the `window_cap` and `protected_cap` fields. This is a fixed cost per cache, not per entry.
- metadata-size (unmeasured): Each map entry now stores a `HashedKey { h, key }`, so every resident key carries an extra 8-byte hash in the hashbrown table. The metadata metric only counts `Node`, so it misses this cost. `get` also clones the key to build the probe, which costs nothing for `u64` keys but would cost something for heap-allocated keys.

## Takeaway

Adding a window and SLRU to the fast-hash cache gave back most of the hit-rate v003 had given up, and did not cost latency. That is because the hypothesis's "hash once" requirement went further than v003's design: the precomputed hash was pushed into the map key itself, not just the sketch. Turning a link from `Option<usize>` into a sentinel-tagged raw index both made room for the region tag and shrank the node. `next` is still an `Option<usize>`, so applying the same `NIL` encoding to it is an obvious next way to cut node size further. Separately, the cost of the 8-byte hash stored in each map entry should be measured, since the current metric doesn't count it.
