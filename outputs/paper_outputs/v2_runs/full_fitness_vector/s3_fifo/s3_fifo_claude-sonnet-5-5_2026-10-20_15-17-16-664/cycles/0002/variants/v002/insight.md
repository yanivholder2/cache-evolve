# Variant v002 Insight

## Helped

- (none) No dimension improved. Metadata-size is unchanged, as expected for a constant-only edit.

## Hurt

- hit-rate: Raising the small-queue fraction from 10% to 15% took slots from the main queue, and the ghost queue is sized from main. Less room for objects that proved themselves outweighed the extra time new objects spent in the small queue. On workload-a this was a slight regression.
- per-op-time: Get and put latencies both moved, but the edit only changes a capacity constant and adds no work to the hot path. This is most likely run-to-run noise, as with the parent's latency shift, and shouldn't be attributed to the ratio.

## Takeaway

On workload-a at size 2048, 15% did not beat 10%, so 10% looks like at least a local optimum on the upper side. Larger small queues cost main-queue capacity faster than they gain re-reference time for new objects. The result comes from a single trace and a single size, and the latency numbers are too noisy to use for comparing sweep points.
