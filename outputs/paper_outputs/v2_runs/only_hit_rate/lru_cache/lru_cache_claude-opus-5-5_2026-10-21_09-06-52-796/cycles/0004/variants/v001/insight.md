# Variant v001 Insight

## Helped

- hit-rate: Cutting `ghost_cap` from `capacity / 2` to `capacity / 4` (at size 2048, from 1024 remembered keys to 512) narrows which ghost hits get promoted straight into `main`. Only keys that come back soon after leaving `small` now skip probation. Keys from longer reuse distances that the parent would have promoted now re-enter `small` and have to earn a place in `main`. That spares the reused residents in `main` from being displaced by keys whose comeback was a weaker sign of reuse.

## Hurt

(No dimension regressed. The fitness vector reports only `hit-rate`. The smaller ghost should halve the `ghost_q`/`ghost_set` footprint and their `with_capacity(ghost_cap + 1)` reservations again, but `metadata-size` and `per-op-time` were not measured, so that gain is unverified.)

## Takeaway

Shrinking the ghost window helped again, so on workload-a at size 2048 the optimum lies at or below `capacity / 4`. Halving the ghost still caught enough genuine re-references to come out ahead. This step gained less than the cycle-3 halving (from `capacity - small_target` to `capacity / 2`), though, which hints that returns are flattening as the window approaches the point where real reuse starts being missed. The next probe on this axis (e.g. `capacity / 8`) should locate the turnover, ideally checked on more than one cache size since this was measured only at size 2048. Then the ghost size can be frozen before structural ghost changes are tried.
