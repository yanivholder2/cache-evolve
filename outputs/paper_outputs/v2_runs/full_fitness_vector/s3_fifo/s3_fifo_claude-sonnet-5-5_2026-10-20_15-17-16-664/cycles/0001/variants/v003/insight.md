# Variant v003 Insight

## Helped

- per-op-time: get and put latencies both dropped versus the parent, but the change adds work (an extra ghost push/pop in `evict_main`) and touches nothing on the `get` path. The improvement is most likely measurement noise or run-to-run variance, not a result of this change.

## Hurt

- hit-rate: Pushing main-evicted keys into the ghost queue and readmitting ghost hits with `freq = 1` slightly lowered hit rate on workload-a. Ghost hits already go straight to main, so the recovered keys displace resident entries that were doing as well or better. The shared ghost, capped at `main_size`, is also now filled by main evictions. That crowds out small-queue evictions, which are the one-hit-wonder signal the ghost exists to track.

## Takeaway

On this trace, recycling main-queue evictions through the ghost queue and giving readmitted keys a head start did not recover premature evictions. It diluted the ghost's role as a filter for small-queue demotions. Main eviction in S3-FIFO appears already well tuned by its freq-decay sweep, so a second chance on top of it costs more than it gains. The latency change should not be credited to this edit.
