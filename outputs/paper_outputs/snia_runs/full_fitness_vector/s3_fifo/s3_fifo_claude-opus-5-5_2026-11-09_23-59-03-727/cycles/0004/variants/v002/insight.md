# Variant v002 Insight

## Helped

- per-op-time: `get` did not change, but the `put` tail got shorter. A likely reason: compaction in `push_ghost()` now triggers at `2 * ghost_cap` instead of `2 * main_size`, so the O(n) `ghost.retain` sweep runs less often. Its cost is spread over more inserts, so fewer `put` calls land on it. Timing has been noisy in earlier cycles, so this reading is plausible but not proven.
- hit-rate (hm_0@16384): The small hm_0 cache gained a sliver. With a longer ghost window, some small-queue rejects that come back after a medium gap are recognised and sent straight to main. At this size main is tight enough that re-admitting them pays off. The gain is so small that it may be noise.

## Hurt

- hit-rate (web_0, both sizes): Making the ghost window longer (capacity instead of `main_size`) lowered web_0 at both sizes, including the targeted 131072 size. More keys that were evicted long ago now count as ghost hits. Ghost hits skip the small queue and force an `evict_main()`. The result is that keys whose earlier reuse was far in the past push out main residents that are still being reused.
- hit-rate (hm_0@131072): The same ghost-hit inflation slightly lowered the large hm_0 cache. This breaks the hypothesis's condition that hm_0 must hold.
- metadata-size: The `S3Fifo` struct grew by one word because of the new `ghost_cap` field. Per-entry size (the summary) did not change. Live ghost memory can also grow by about a third, since the bound rose from about 0.75× to 1.0× capacity, but this metric does not measure that.

## Takeaway

The paper sizes its ghost queue by the cache's entry count. Copying that here does not help: on this implementation, remembering more history lets stale reuse into main, where it displaces entries that are still being reused, and web_0 at the large size gets worse instead of better. This rules out ghost length as the cause of web_0's remaining loss. Following H02's own fallback, the next probe should be the small-queue share (`capacity / 4`), and the ghost bound should go back to `main_size` (or stay no larger).
