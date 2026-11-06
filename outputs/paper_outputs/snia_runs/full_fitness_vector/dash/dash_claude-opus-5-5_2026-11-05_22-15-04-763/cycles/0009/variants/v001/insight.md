# Variant v001 Insight

## Helped

- per-op-time (`put` p50, p99): Both put percentiles dropped well below the parent, and the median and tail moved by about the same amount. This fits the change: a demotion or eviction that removes a stash item now clears its mask bit with one test of `stash_collide` and one AND-NOT, and calls `refresh_stash_mask` only when a bit collision was recorded. So the remaining per-removal re-hash loop, which the parent's insight called "a much smaller remaining cost", was still a large part of put time, and with only a few items per bucket across 64 bits it is almost always skipped.
- per-op-time (`get` p50): The median get also got faster, even though the read-side mask test and the `get_position` scan are unchanged from the parent. The most likely cause is the stash-hit promotion path in `promotion_get`. It reuses its precomputed hash to clear the bit in O(1) instead of rebuilding the mask. That is common enough on these traces to move the median, not just the tail.

## Hurt

- per-op-time (`get` p99): The get tail got somewhat worse, though put's tail improved. The O(1) path does not cover everything: when a collision bit is set, removal still falls back to `refresh_stash_mask`, which now rebuilds two words per pass. Every stash insert and removal also touches a second parallel u64 array, which can mean another cache line on the slow promotion path. The promotion item-shift was deliberately left alone, so whatever dominates the get tail now is the shift plus these extra accesses, not mask re-hashing. With one sample per variant, part of this move may be noise.

(hit-rate is bit-identical to the parent on every trace and size. That is the expected result, because only mask bookkeeping changed and the mask still never under-reports. metadata-size is unchanged because `stash_collide` lives in the segment's heap storage next to `stash_masks`, not in `Item` or the `Dash` handle.)

## Takeaway

Making stash-mask clears O(1) with a conservative per-bucket collision word, while keeping the dense read-side mask exactly as it was, gives the put recovery that 0008 v001/v002 achieved without their `get` p50 regression. This confirms that the read structure, not O(1) upkeep itself, caused those regressions. The rise in the `get` tail suggests that the next cost to target on stash-hit promotions is the item shift and the collision fallback, not mask maintenance.

(Note: In this session I could open only the overlay's `src/shared/utils.rs`, which holds the hasher and is carried over unchanged from the parent. I could not list or open the segment source, so the claims about how `stash_collide`, the removal fast path and the refresh fallback are implemented come from the hypothesis.)
