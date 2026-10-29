# Variant v001 Insight

## Helped

- hit-rate: Both traces gained at the 16K size, where the ghost is on. Keys that come back soon after being evicted from probation now get `SC_BIT` set and survive one more probation eviction. At 131K the ghost is an empty slice, so every ghost step is skipped and the results match v002 exactly. That is the intended effect of the size gate: it avoids the large-cache hm_0 loss seen in v003.
- per-op-time (get p99): Get p99 fell back down. `get` and `on_hit` are almost unchanged from v002, so this most likely reverses the parent's p99 spike rather than being a real gain. It supports the parent insight's suspicion that the spike was noise.

## Hurt

- per-op-time (put): Put p50 and p99 both rose sharply. With the ghost on, an evicting put now hashes the new key twice (`hash_one` and then `map.entry`) and the victim's key twice (`hash_one` for its ghost tag and then `map.remove`). That is four hash computations where v002 needed two, which undoes v002's single-probe gain. The cost also lands on puts that update an existing key, because the ghost and reprieve work runs before `entry` tells resident and new keys apart.
- per-op-time (get p50): Get p50 regressed, which the hypothesis said it would avoid. Moving the flag into `prev` kept `next` unmasked, but `set_protected(true)` became an OR plus an AND-NOT. It runs on every protected hit and every promotion, so `on_hit` is not identical to v002's. A second likely cause is the extra memory a get competes with: a ghost of up to 64 KB, plus the header growing to exactly the 128 B limit.
- metadata-size: `LruCache` grew by the `Box<[u16]>` fat pointer and now sits exactly at the 128 B ceiling, with no room left for later header fields. `Node` stays at 24 B.

## Takeaway

Moving the reprieve bit into `prev` and turning the ghost off for large caches kept v003's hit-rate gain at small sizes and removed its large-size loss. However, it moved the cost onto `put`: computing ghost tags with a separate `hash_one`, alongside the map's own hashing of the same keys, doubles the hash work of an evicting put. Also, adding even a single AND to `set_protected` still showed up in get p50. A follow-up should compute the ghost tag from the hash the map already computes (for example, by keying on a precomputed hash through the raw-entry or `hashbrown` API). It should also run the ghost work only on the vacant path, and keep `set_protected(true)` a single OR (clearing `SC_BIT` at reprieve or eviction time instead).
