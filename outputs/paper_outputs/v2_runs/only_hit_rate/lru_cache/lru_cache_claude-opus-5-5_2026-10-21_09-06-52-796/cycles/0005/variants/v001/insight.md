# Variant v001 Insight

## Helped

- hit-rate: Halving `ghost_cap` from `capacity / 4` to `capacity / 8` means that at size 2048 only the last 256 keys evicted from `small` are remembered, down from 512. That is now only a little larger than `small_target` (204). So the ghost-hit fast path into `main` only fires for keys that return within about one more pass through `small`. Keys with longer reuse distances re-enter probation, and on workload-a they apparently still don't pay for the `main` residents they would push out. Shrinking the ghost also tightened the filter in the same direction as before, and the result still came out slightly ahead.

## Hurt

(No dimension regressed. The fitness vector reports only `hit-rate`. The smaller ghost halves the `ghost_q`/`ghost_set` footprint and their `with_capacity(ghost_cap + 1)` reservations once more, but `metadata-size` and `per-op-time` were not measured. The in-file tests moved from capacity 10 to 16 so they still check `ghost_cap == 2`. Their meaning didn't change, and they have no bearing on fitness.)

## Takeaway

The probe that was meant to find where the curve turns over didn't find it. `capacity / 8` still beats `capacity / 4` on workload-a at size 2048, so the ghost size can't be frozen at `/4` as the hypothesis planned for that outcome. The gain was clearly smaller than the previous halving, though. At 256 keys the ghost window is close to the size of `small` itself, so a `capacity / 16` probe would make the ghost smaller than the probation queue. That probe tests whether the ghost-into-`main` path is worth having at all on this trace. It should also be checked at a second cache size, because gains this small, from a single trace and size, could just reflect that trace's particular reuse-distance pattern rather than a general property.
