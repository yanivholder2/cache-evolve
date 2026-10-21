# Variant v002 Insight

## Helped

(No measured dimension improved. The fitness vector still contains only `hit-rate`, and H02 was a cost-only change aimed at metadata-size, which isn't evaluated. The overlay does the full refactor with no fallback: `Item<K,V>` now holds only `key` and `value`, and the in-crate `item_is_compact` test asserts `size_of::<Item<u64,u64>>() == 16`. So the per-item win holds by construction but hasn't been measured.)

- hit-rate (held, not improved): unchanged, as H02 predicted. The GCLOCK counter moved from `item.lfu_counter` into the packed per-bucket `clock_bits` word. Every point that touches it (hit increment in `get_from_position`, the decrement sweep in `clock_find_victim`, admission on the replace path and the push path, and the `ctr_remove` shift that mirrors `Vec::remove`) now goes through `ctr`/`set_ctr`, and these keep each counter in the same slot as its item. That is the result H02 named as proof that the bookkeeping did not drift, especially on the `remove_at` shift path.

## Hurt

(No measured dimension regressed.)

## Takeaway

All per-item replacement metadata can move out of `Item` and into positional per-bucket storage without changing the GCLOCK/LIP victim sequence. The one risky piece is the counter shift that must accompany `Vec::remove`. The packed-word shift (`low | high`, with guards at slot 0 and at the top slot) handles it correctly, and in-crate tests cover both edges. H02 expected the bucket to cost only 4 extra bytes, but the actual cost is larger. The implementation uses a `u64` word (up to 32 slots) instead of a `u32`, and it adds two `Vec` headers to every bucket even when they are empty: `meta` for non-clock policies and `clock_wide` for buckets wider than 32 slots. That is roughly 56 extra bytes per bucket. On a 16-way bucket the 8 bytes saved per item still come out well ahead, but the net per-entry saving is smaller than "24 → 16" suggests. A future cut could make `meta` and `clock_wide` conditional, for example with an enum, so that clock mode carries only the one counter word.
