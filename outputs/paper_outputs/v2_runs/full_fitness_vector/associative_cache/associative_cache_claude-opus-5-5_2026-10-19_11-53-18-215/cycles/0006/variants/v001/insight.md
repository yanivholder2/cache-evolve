# Variant v001 Insight

## Helped

- hit-rate: The protected cap went from 3/4 to 7/8, with a probation floor that scales with bucket size. Proven-reused keys now get more of each merged bucket, and an 8-slot bucket protects 7 items instead of 6. Hit rate landed exactly on the value cycle 0005 v002 reached with the same cap on the 24-byte item, and exactly on this hypothesis's prediction. So the hit-rate gain comes entirely from the cap, and `Item` width plays no part in it.

## Hurt

- per-op-time (get): The SLRU path is positional, so a protected hit runs `items[pos..].rotate_left(1)` across the protected segment, and that segment is now longer. Get got slower at both the median and the tail. The narrower 16-byte stride did not absorb the cost.
- per-op-time (put): Put slowed most at p99. With the protected segment one slot larger, the tail branches move more data: the ghost-hit `insert(len0)`, `evict_front`'s `remove(0)`, and the resident-update rotate each shift a longer run. The 8-slot bucket also has a 1-slot probation, which likely makes evictions and ghost re-inserts more frequent, so those whole-bucket memmoves run more often.

## Takeaway

The two axes from cycle 0005 combined cleanly on hit rate and metadata. The 7/8 cap and the 16-byte `{ key, value }` item stacked with no interference, and the merge kept the shrunken item. However, this variant shows that v002's slowdown was real and not an artifact of the 24-byte stride. A 7/8 cap makes protected-side rotates and shifts longer and leaves almost no probation buffer. That cost survives even a third narrower element, so hit rate and per-op latency now trade off directly through the protected-cap ratio.
