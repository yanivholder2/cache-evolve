# Variant v001 Insight

## Helped

- hit-rate: The lower ghost-hit shrink floor, now `capacity / SMALL_FLOOR_DIV` (= /64) instead of `capacity / 16`, lets repeated ghost hits on workload-a give more slots to `main`. Hit-rate landed on the value H01 predicted from cycle 0005 v001, so the floor's gain does not depend on the slab layout. It carries over to the slab-free 16-byte layout unchanged.

## Hurt

- per-op-time (`get`, the summary): `get` is byte-identical to the parent's (one `index.get_mut` plus a freq bump), and `index` is still sized to `capacity`. The code change therefore has no direct path to `get` cost. The regression, with p99 moving far more than p50, points to the measurement environment or to a residency or cache-line effect of the different resident set, not to a code-path cost. This variant cannot tell those apart.
- per-op-time (`put`): Unlike `get`, the policy change does reach the `put` path. With a smaller `small` and a larger `main`, more ghost hits go into the ghost-hit branch, which runs `evict_small_inner(false)` and `evict_main` fix-ups. `evict_main` also does its CLOCK walk over a longer `main` that holds more keys with nonzero freq, so it makes more hash probes per eviction. That can plausibly raise `put` cost, especially the tail. How much of the regression is real, rather than the same noise that affected `get`, is unclear.

(metadata-size did not move. This was expected: `Entry` is still 16 bytes, the `entry_is_16_bytes` test is kept, and the only change outside tests is a `const` plus an inline method, so no fields were added to `S3Fifo`.)

## Takeaway

The small-queue floor and the storage layout really are independent levers. Putting the /64 floor onto the slab-free layout reproduced the slab variant's hit-rate exactly, and metadata-size stayed at the minimum, so this variant dominates both cycle-0005 incumbents on those two axes. The latency numbers moved even though `get` is unchanged, which shows that per-op-time noise between runs is large enough to hide or fake small effects. Per-op-time should be re-measured alongside the parent before anyone concludes the floor itself costs latency.
