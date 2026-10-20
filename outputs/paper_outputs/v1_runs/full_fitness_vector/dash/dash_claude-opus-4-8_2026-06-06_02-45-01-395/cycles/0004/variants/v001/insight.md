# Variant v001 Insight

## Helped

- metadata-size: Deleting `Item::lfu_counter` and relocating the promotion heat into the segment-owned `target_heat` side table removed the trailing-`u8` alignment padding, shrinking `Item<u64,u64>` from 24 to 16 bytes and restoring the metadata floor.

## Hurt

- per-op-time: Replacing the inline `lfu_counter` field access with a `HashMap` `entry`/`remove` against `target_heat` on the hot target path traded a predictable struct read for hashing-and-probing variance, raising put p50 and pushing both get and put p99 tails up.

## Takeaway

Moving the `PROMOTE_AT` heat signal out of `Item` and into a segment-side `HashMap` reaches the previously-open Pareto corner — the best hit-rate (preserved bit-for-bit, since the gate decisions are unchanged) AT the 16-byte `Item` floor that the counter-carrying parent could not hold. The cost is that a side-table lookup is not a free substitute for an inline field: the struct-of-arrays relocation buys size by spending tail latency on hashing.
