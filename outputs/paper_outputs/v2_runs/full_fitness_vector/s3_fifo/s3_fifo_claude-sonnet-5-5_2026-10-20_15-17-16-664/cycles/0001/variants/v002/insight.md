# Variant v002 Insight

## Helped

- hit-rate: Shrinking the small queue from capacity/4 to capacity/10 gives the main queue more slots. The ghost queue is sized to main_size, so it also grows. Together these keep more re-referenced objects resident, and the gain is modest on workload-a.
- per-op-time: The change touches only a constant, so no new work was added to get/put. The large drop in get latency is probably measurement noise or a run-to-run difference rather than an effect of the smaller small queue. Don't credit it to the change without a re-run.

## Hurt

- (none) metadata-size is unchanged. Only the split of a fixed capacity moved, so the struct and entry footprint stay the same.

## Takeaway

Moving the small-queue ratio from 25% to the paper's recommended ~10% gave a small hit-rate gain on a single LIRS trace at size 2048. This suggests that S3-FIFO's small/main split is a cheap tuning knob that is worth sweeping per workload. The latency change has no mechanism behind it, so it shouldn't be read as a result of this edit.
