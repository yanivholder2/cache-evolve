# Variant v001 Insight

## Helped

- hit-rate: Raising `MAX_FREQ` from 3 to 7 lets a key that gets hit often while in `main` build up to 7 retention credits instead of 3. `evict_one` takes one credit each time the key reaches the front of `main`, so the hottest keys now survive up to 7 rotations instead of 3. On workload-a this gave only a marginal gain, too small to show the two levers adding up.

## Hurt

(No measured dimension regressed. The fitness vector reports only `hit-rate`. The cost of this change is mixed into the small net gain: a key that was hot and then went cold now keeps its slot for up to 7 extra rotations before eviction, instead of 3. Those slots are taken from keys whose reuse is current. That offsets much of the benefit the extra credits give to keys that are still hot.)

## Takeaway

The hypothesis said a hit-rate near v003's would mean the two levers overlap, and that is roughly the result: v003's `small = capacity/20` split had already captured most of what deeper retention could add. One likely reason is that credits can only build up while a key is in `main`, since both promotion from `small` and ghost admission reset `freq` to 0. So the higher ceiling only matters for keys hit more than 3 times between their own `main` passes, which is rare once `main` is already large. Retention depth looks close to used up as a lever in this configuration. It is worth keeping `MAX_FREQ = 7` because it costs nothing, but further gains will probably have to come from changing how credits are earned or carried over, not from raising the cap again.
