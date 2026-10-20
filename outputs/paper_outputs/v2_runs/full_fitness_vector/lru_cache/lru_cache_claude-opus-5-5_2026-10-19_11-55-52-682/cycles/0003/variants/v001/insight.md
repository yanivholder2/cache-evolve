# Variant v001 Insight

## Helped

- metadata-size: Removing `prev: u32` from `Node` drops the node into the next smaller alignment bucket. `visited: bool` and the remaining `next: u32` now share one 8-byte word after `key`/`value`, so each node loses a full word, not just 4 bytes. The new `hand_prev: u32` on `LruCache` costs nothing because it fills the padding that the three `u32` cursors (`head`/`tail`/`hand`) left before `capacity: usize`. So the per-node saving is pure gain, and the cache struct does not grow.

## Hurt

- per-op-time: Nothing measurably regressed, and nothing measurably improved either. `get` and in-place `put` were already relink-free in the parent, so dropping the back-link does not affect them. On eviction, removing one `prev` write in `unlink`/`push_back` is offset by carrying `p = h` through every step of the hand walk in `find_victim`. The locality benefit that the rationale expected from smaller nodes did not appear on this trace and capacity.

## Takeaway

SIEVE only ever removes the node under the hand and only appends at the tail. That makes the doubly-linked queue's back-pointer redundant: a predecessor carried by the hand replaces it at zero per-node cost, and hit-rate is unchanged because eviction order is bit-for-bit the same. This makes the change a near-free representation win for metadata, but it should not count as a speed lever. Once hits stop relinking, the eviction path is too infrequent for a lighter node to show up in per-op latency.
