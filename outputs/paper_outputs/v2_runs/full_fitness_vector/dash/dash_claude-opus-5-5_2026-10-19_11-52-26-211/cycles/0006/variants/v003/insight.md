# Variant v003 Insight

## Helped

- hit-rate (marginal, within noise): The change is implemented exactly as specified. A native victim of a target insert now fills a free slot in its probing bucket `(source_idx + 1)` at the LRU end before trying the stash. It is guarded by `NATIVE_DEMOTE_TO_PROBING_HOLE`, and the foreign-branch cascade is unchanged. Hit rate rose by one trace hit's worth at most, which falls short of the "clearly above v001" bar in H03. On this trace, native victims almost never find a probing hole when they leave target.

## Hurt

- per-op-time (get p50, noise-level): The new branch adds one `is_full()` check on the neighbouring normal bucket to the stash-hit and hottest→probing paths whenever a native victim is pushed out. The tiny rise in `get` p50 fits that extra check: the branch runs but almost never takes the hole, so the check is cost with no benefit.
- per-op-time (put p99, weak): `put` itself did not change; it still goes straight to `stash_insert`. The p99 shift is therefore probably noise. It may also be an indirect effect: any rescued items make probing buckets fuller, so later hottest→probing promotions evict, and then cascade, more often. `put` p50 did not move.

(metadata-size did not change: no fields were added, and `Item` stays at 40 bytes.)

## Takeaway

Free slots in a native victim's probing bucket are almost always gone in steady state at the moment the victim is demoted. So "probing holes left by `remove_at`" is not a real idle-capacity source on this workload. This supports the refutation branch of H03: the holes are rare, or hottest→probing promotions fill them first. Native-demotion rescue into probing does not raise hit rate. If v003 is kept, it only adds a branch on the hot paths. It should not be stacked with H01-style stash-victim rescue. Idle capacity must be found somewhere other than the target's probing neighbour.
