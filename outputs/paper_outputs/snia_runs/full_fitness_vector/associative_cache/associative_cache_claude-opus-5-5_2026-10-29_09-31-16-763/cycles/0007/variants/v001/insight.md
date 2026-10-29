# Variant v001 Insight

## Helped

- hit-rate: Not a gain, but the invariant held. All four cells match the parent exactly. The inline `rotate_left(1)` on `items[position..]` and `tags[position..]` gives the same order as ClassicLRU's `remove(position)` + `push`, and ClassicLRU writes no `Item` field on a hit. The lockstep reference test and the new MRU-end/front-hit test both check this.
- metadata-size: Unchanged, as intended. The inline path is gated on the existing `lip && victim_is_front` flags, so no struct field was added and the bucket stride stays the same.

## Hurt

- per-op-time: Get got clearly slower at both p50 and p99, which is the opposite of what the hypothesis predicted. The only change on the get path swaps `Vec::remove` + `push`, which are plain `memmove`s, for two `slice::rotate_left(1)` calls. The likely explanation is that the standard library's `ptr_rotate` is not one memmove for short slices. For a short suffix (about 24 elements or fewer), it uses a scalar, index-juggling loop that moves one element at a time with modular index arithmetic. That loop runs once for the 24-byte `Item`s and again for the `u8` tags. Hits under LIP land throughout the bucket, so this slow path runs on almost every hit. It costs more than the removed virtual call and len/capacity bookkeeping saved. Skipping the move for MRU hits did not make up the difference.
- per-op-time: Put p50 and p99 also rose, even though the miss path (`fused_evict_front_insert`) is byte-for-byte the parent's. The update-on-hit branch of `put` now also goes through the rotation, which explains part of it. The rest looks like the same layout/inlining effects or tail noise the parent's insight flagged, and should be re-measured before anyone attributes it to the rotation.

## Takeaway

On this hit path, `rotate_left(1)` is not a cheaper way to write remove+push. The memmoves were never the overhead. The generic rotate algorithm is the overhead for short suffixes, and it outweighs the devirtualisation it came with. A follow-up should keep the inline, devirtualised, MRU-skip structure. It should do the move by hand: `ptr::read` the item at `position`, then `ptr::copy` `items[position+1..n]` down one slot, then `ptr::write` the item at `n-1`. The tags should get the same treatment with `copy_within`. This mirrors the `fused_evict_front_insert` pattern that already won on put.
