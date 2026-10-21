# Variant v001 Insight

## Helped

- hit-rate: Cutting `ghost_cap` from `capacity / 8` to `capacity / 16` means that at size 2048 the ghost holds 128 keys, which is now fewer than `small_target` (204). A key evicted from `small` therefore only skips probation and goes straight into `main` if it comes back within roughly half a pass through `small`. Narrowing the fast path this much did not cost hit-rate on workload-a. It came out marginally ahead, by a margin small enough to count as flat. So the keys this change drops from the fast path, those with reuse gaps between about 128 and 256 evictions, were not earning their place in `main` anyway. Sending them back through probation, where they have to earn promotion with a `freq > 0` hit, works just as well.

## Hurt

(No dimension regressed. The fitness vector reports only `hit-rate`. The ghost footprint (`ghost_q`, `ghost_set` and their `with_capacity(ghost_cap + 1)` reservations) halves again, but `metadata-size` and `per-op-time` were not measured. The in-file ghost tests moved from capacity 16 to 32 so they still check `ghost_cap == 2`. Their asserts now expect `small_target == 3`. They test the same behaviour as before and have no bearing on fitness.)

## Takeaway

The probe was built to settle the question, and it did. It hit the branch the hypothesis set out in advance: "goes up or stays flat". A ghost window smaller than the probation queue loses nothing on workload-a at size 2048. The gains from successive halvings have shrunk to almost nothing, which suggests the ghost-into-`main` path adds very little on this trace. The probation queue's own `freq > 0` promotion in `evict_one` appears to be doing the real admission filtering. The next step is to remove the ghost-hit fast path entirely, or set `ghost_cap` to 1, and check that result at a second cache size. Results this flat from a single trace and size cannot tell "dispensable" apart from noise.
