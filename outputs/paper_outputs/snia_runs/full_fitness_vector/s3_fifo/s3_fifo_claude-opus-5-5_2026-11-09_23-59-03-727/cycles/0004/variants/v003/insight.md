# Variant v003 Insight

## Helped

- metadata-size: The fixed `S3Fifo<u64,u64>` struct got smaller. Dropping `ghost_index` removed one `HashMap` header, and the new `ghost_live: usize` counter adds back only part of that. The summary metric, the per-entry `Entry<u64,u64>` footprint, did not change, as the hypothesis predicted.

## Hurt

- per-op-time: The `get` median went up a little instead of down. `get` already did only one probe, so the merge saved it nothing. Every resident lookup now searches a table that also holds all live ghost keys. Each bucket is also wider: `(u64, Slot)` is 24 bytes because the `Slot` enum carries a `u64` payload plus a tag, while the parent's `(u64, u32)` bucket was 16 bytes. So each probe touches more cache lines.
- per-op-time: The `put` median rose, and the `get` and `put` tails grew by more than an order of magnitude. The likely cause is load factor. `with_capacity(capacity + main_size)` asks for 1.75× capacity, and ×8/7 that lands exactly on a power-of-two bucket count at both cache sizes. Once residents and ghosts both fill up, the merged table runs near hashbrown's 7/8 maximum load. The parent's two tables ran at about 1/2 and 3/4 load. At that load, misses have to scan long probe chains before they find an EMPTY slot. Ghost-bound `remove`s leave tombstones that make this worse. Occasional rehash or grow events are also likely. One fewer probe does not make up for that.

(hit-rate did not move: it is identical to the parent on every trace and size. That confirms the `PENDING` marker and the bound-before-push ordering kept policy decisions unchanged.)

## Takeaway

Halving the hash probes per miss/ghost-hit did not pay off. Merging the two indexes also merged their occupancy: one table holding residents and ghosts, sized at exactly `capacity + main_size`, runs near maximum load with wider buckets. That cost more per probe than the removed probe saved, and it hurt the tails most. A unified index would need generous headroom (for example, sizing for about 2× the live entries, or a compact `Slot` encoding such as a `u32` slab index with a high-bit ghost flag) before the probe reduction can show up.
