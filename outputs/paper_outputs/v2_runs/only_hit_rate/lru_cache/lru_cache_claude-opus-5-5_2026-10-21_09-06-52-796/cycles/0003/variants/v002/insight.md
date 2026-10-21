# Variant v002 Insight

## Helped

- hit-rate: Raising the `small`→`main` promotion bar from `freq > 0` to `freq >= PROMOTE_FREQ` (2) moved hit-rate up by a negligible amount, too small to support the claim that a tighter filter helps. The large ghost undoes most of the tightening. A key hit once in `small` is now evicted, but `push_ghost` keeps it, and the ghost holds about `capacity - small_target` keys. If the key comes back while ghosted, the ghost-hit path in `put` sends it straight into `main`. So in most cases the change only delays admission by one miss. It rarely keeps a key out.

## Hurt

(No dimension regressed. Only `hit-rate` was measured, on one trace at one size. `per-op-time` and `metadata-size` were not measured. This change should leave them roughly unchanged: it swaps one comparison and adds no state.)

## Takeaway

In v001's S3-FIFO, how hard promotion from probation is and how big the ghost is are not separate settings. With a ghost of about 90% of capacity, the extra resident hit that v002 requires is mostly replaced by a ghost re-admission, so `PROMOTE_FREQ` barely moves hit-rate on workload-a. Testing whether stricter admission really helps would mean changing the ghost path as well. For example, shrink `ghost_cap`, or make a ghost hit re-enter `small` instead of going to `main`. Raising the probation threshold alone does not test it.
