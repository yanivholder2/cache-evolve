# Variant v003 Insight

## Helped

- hit-rate: On buckets of 16+ slots, the protected cap went from `max_size - 2` to `max_size - max(1, max_size/16)`, so a 16-slot merged bucket now protects 15 reused keys instead of 14. Hit rate moved, and replay of a fixed trace is deterministic, so the factory does build buckets of at least 16 physical slots. This variant was not the planned no-op, and the policy, not the geometry, is still the lever. The gain is real but much smaller than the earlier 3/4 → 7/8 step. On this trace, the curve is still rising above 7/8 but is clearly flattening.
- per-op-time (get): Get did not slow down, which goes against the rationale's expectation that a longer protected segment costs latency. The protected-hit `rotate_left(1)` over `items[pos..]` grows by at most one element per 16 slots, which is negligible. The higher hit rate means a few more lookups return early on a match instead of scanning the whole bucket. The small improvement is within plausible run-to-run noise, so read it as "no cost", not as a speedup.
- per-op-time (put): Put's tail improved. With `len0` usually 1 on a full 16-slot bucket, `mid = 0`, so a cold key takes the in-place `mem::replace(items[0])` path plus a 1-element rotate instead of shifting a 2-slot probation. The parent's 1-slot-probation churn at 8 slots is unchanged, so this variant adds no new memmove-heavy path.

## Hurt

- per-op-time (put): Median put is essentially flat to marginally worse. A 1-slot probation on 16-slot buckets means every cold miss evicts the previous newcomer immediately, so ghost-ring writes and `take_ghost` scans run on nearly every put. That is the likely reason the median did not improve along with the tail.

## Takeaway

Taking protected from 7/8 to 15/16 on 16-slot buckets still buys hit rate, but only a sliver: diminishing returns are setting in. Unlike the parent's 3/4 → 7/8 step, it did so with no latency or metadata penalty, because it only shifts one slot per 16 and reuses the existing `mid = 0` replace-in-place path. The protected-cap knob is nearly exhausted. Squeezing probation down to a single slot leaves cold keys only one miss of residency, so further hit-rate gains will probably need a smarter admission or ghost policy rather than a larger protected share.
