# Variant v002 Insight

## Helped

- metadata-size: Moving `next`, `visited` and `small` out of `Node` into the packed `links: Vec<u32>` leaves `Node<u64,u64>` as just `{ key, value }`. The SMALL flag fits in bit 30 next to VISITED, so S3-FIFO's extra per-node state costs nothing beyond the 4-byte link word that SIEVE already needed. This brings the hit-rate leader down to the same per-node footprint v001 reached on plain SIEVE.
- hit-rate: Held exactly, which is the intended result rather than a gain. Every queue operation was rewritten through the `next`/`set_next`/flag helpers without changing the order of operations, ghost fingerprinting still happens inside `evict()` before the key swap, and `small_target`, the ghost and the hasher were left alone. Eviction order therefore matches v003.

## Hurt

- per-op-time: `get` used to be one map probe followed by a single 24-byte `Node` access that both set `visited` and read `value`. It now writes `links[idx]` and reads `nodes[idx].value`, which live in two separate arrays, so a cold `get` touches two cache lines instead of one. That fits `get` getting slower at both p50 and p99. The hypothesis expected the side array to help `put` p99, as it did on v001, but that did not happen here. S3-FIFO's `put` already pays for a `hash_one`, a random ghost probe and, on a small-head eviction, a second `hash_one` on the victim. These costs appear to swamp the savings from the key swap and the tight `find_victim` loop, and the miss path also now writes to the separate `links` array.
- metadata-size (struct): `LruCache<u64,u64>` grew because the new `links` `Vec` adds another header (pointer, capacity, length). Per-node bytes are what the summary tracks, but the per-instance fixed cost went up. The `links` heap buffer (4 bytes per slot) now holds the bytes that were removed from `Node`, so this saves less total memory than the per-node figure suggests.

## Takeaway

Packing S3-FIFO's link, visited and small-queue state into a `u32` side array is a clean memory win: per-node size drops by a third, eviction order is unchanged, and the SMALL bit costs nothing extra. The cost is that a `get` now has to touch both the flags and the value, which were previously on the same cache line, so it pays a split-array penalty. The `put`-tail benefit seen on SIEVE also does not carry over to S3-FIFO's heavier miss path. Future variants should treat the side array as trading `get` latency for memory, not as a free change. If `get` latency matters, one option is to keep VISITED in `Node` (or keep value and flags together) and move only the links out.
