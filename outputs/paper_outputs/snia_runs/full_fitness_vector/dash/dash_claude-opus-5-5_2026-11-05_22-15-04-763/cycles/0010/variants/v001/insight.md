# Variant v001 Insight

## Helped

(None. No dimension improved on the parent. hit-rate is bit-identical across all four trace/size points, as expected: the change only touches mask bookkeeping, and the no-false-negative invariant keeps stash-first lookup results the same. metadata-size did not move because the new `stash_collide` words live in the segment, and neither measured type, `Item<u64,u64>` or the `Dash` handle, holds them.)

## Hurt

- per-op-time (`put` p50): Put got clearly slower instead of moving toward 0009 v001's figure. Every stash insert now does a read-modify-write on a second per-bucket word (`stash_collide[i] |= stash_masks[i] & m`) on top of the mask update. Every stash removal adds a load of `stash_collide[i]`, a `count_ones()` and a threshold branch. Laid out as a parallel array, the collide word likely sits on a different cache line from the mask, so each stash-touching put pays an extra line. This extra fixed cost outweighed whatever was saved by skipping `refresh_stash_mask` on ordinary removals.
- per-op-time (`put` p99): The put tail got much worse, the biggest regression in this variant. The collide word only accumulates: bits are set on insert and cleared only by a full rebuild. Inserts also take collisions from `stash_masks`, which already holds stale bits, so stale bits create new collide marks and the popcount climbs toward `COLLIDE_REFRESH` (6) on a steady schedule. When it crosses, `refresh_stash_mask` now rebuilds two words in one pass, so the occasional rebuild costs more than the parent's per-removal rebuild did, and it lands in the tail.
- per-op-time (`get` p99): The get tail drifted slightly up. This matches stale collided bits staying set in `stash_masks` between lazy rebuilds, which raises the stash filter's false-positive rate a little and sends a few more absent keys into the `get_position` stash scan. `get` p50 did not move, so the effect is confined to the tail.

## Takeaway

Per-bit collision tracking made stash removal O(1) in the common case, but the extra collide word that every stash insert and removal must touch cost more than the occasional `refresh_stash_mask` it avoided. With k=2 bits per key, about 8 keys per bucket and a 64-bit word, collisions are common, and stale bits create more collide marks. A popcount-6 lazy trigger therefore fires often and turns the rebuild into a put-tail spike instead of removing it. A cheaper O(1) clear for k=2 probably needs to avoid storing a second word on the hot path. One option is to rebuild only on stash LRU eviction and accept stale bits on promotion. Another is to keep 0009 v001's simpler k=1 clear for writes.

(Note: I could open the overlay's `src/shared/utils.rs`. It confirms that `stash_mask_bits` is unchanged, which the hypothesis required. I could not list or open the segment source that holds `stash_collide`, the insert/remove paths or `refresh_stash_mask`. The claims about memory layout and how often the refresh fires are inferred from the hypothesis's specified changes and the direction of the measured deltas, not from reading the implementation.)
