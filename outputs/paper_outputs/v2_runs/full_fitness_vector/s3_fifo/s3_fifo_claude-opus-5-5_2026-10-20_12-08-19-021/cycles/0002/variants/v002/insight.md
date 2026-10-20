# Variant v002 Insight

## Helped

(none — `metadata-size` didn't move. `Entry<K,V>` and the `S3Fifo` struct layout are unchanged; only the value of `ghost_cap` changed, and the ghost's larger heap buffer doesn't count toward the per-entry summary.)

## Hurt

- hit-rate: The hypothesis expected a gain, but workload-a came out slightly worse than v003. Now that every freq-0 main victim is ghosted, close to one key enters the ghost per miss, not just the small-queue rejects. Raising the cap from `main_size` to `capacity` doesn't keep up with that, so small-queue rejects stay in the ghost for a shorter stretch of the trace. Main victims, meanwhile, are keys that already used up all their second chances, and a ghost hit now puts them straight back into main without going through the small-queue filter.
- hit-rate (adaptation side effect): Ghost hits on main victims also trigger v003's "grow main" step. That probably pushes `small_size` toward its `capacity/16` floor and weakens the small queue's job of filtering one-hit keys. The eval output doesn't log `small_size`, so this is unconfirmed.
- per-op-time: The `put` slowdown is explained by the diff. Every miss in `put` scans the ghost with `iter().any`, and every ghost hit runs a full `retain` over it. The ghost is now about a third longer, and ghost hits are more frequent, which means more `retain` passes and more fix-up `evict_main` calls. The `get` slowdown is *not* explained, because `get` doesn't touch the ghost and its code is unchanged. It most likely cancels v003's own unexplained `get` improvement and shouldn't be blamed on H02.

## Takeaway

On workload-a, adding main-queue victims to the ghost does not help. Victims that already failed the second-chance check are poor candidates for going straight back into main. They also dilute the ghost history that the small-queue admission and the ARC-style split depend on. With linear-scan ghost lookups, the bigger ghost also costs `put` latency, so H02 loses on both dimensions. A retry would need a separate main-victim ghost, or one that doesn't feed the split adaptation, plus a hashed ghost index so the latency cost can be judged on its own.
