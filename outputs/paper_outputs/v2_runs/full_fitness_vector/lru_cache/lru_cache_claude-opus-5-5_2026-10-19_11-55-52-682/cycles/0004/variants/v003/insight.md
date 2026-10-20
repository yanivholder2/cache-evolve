# Variant v003 Insight

## Helped

- metadata-size (`Node<u64,u64>`): Moving `next` and `visited` out of `Node` into a parallel `links: Vec<u32>` leaves `Node` as just `{ key, value }`, so the 8-byte word that held a 4-byte link plus a padded bool is gone. Packing the visited flag into bit 31 of the link means the side array costs 4 bytes per entry, not 8. The real per-entry footprint across both arrays is therefore 20 bytes against the parent's 24. The headline node figure shows a larger cut only because it no longer counts the policy state.

## Hurt

- metadata-size (`LruCache<u64,u64>`): The new `links` field is a second `Vec` header (pointer, length, capacity) on the cache struct. Unlike v001's `hand_prev`, it could not fit into existing padding, so the fixed per-cache cost grows. This is a constant cost and does not scale with entry count.

## Takeaway

Moving SIEVE's link and visited bit into a packed side array of `u32` removes all policy state from `Node`. Hit-rate is unchanged because `find_victim` runs the same walk over the same order. The change saves 4 real bytes per entry and adds one `Vec` header per cache. The node-size metric overstates the gain because it no longer sees `links`, so future comparisons should count node plus side-array bytes per entry. On per-op-time, the second memory touch on each hit and the denser hand sweep both came out roughly neutral, so this layout is a memory lever, not a speed lever.
