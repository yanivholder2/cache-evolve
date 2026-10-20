# Variant v003 Insight

## Helped

- hit-rate: Shrinking the small queue to 5% of capacity gives the main queue more room, so objects that earn promotion stay resident longer. Workload-a rewards that.
- per-op-time: The change was a single constant, so it adds no work to the get or put path. The large latency drop is most likely measurement noise or a smaller small-queue scan. The constant can't plausibly account for it.

## Hurt

- metadata-size: No movement. A constant change doesn't alter struct layout.

## Takeaway

Reducing the small-queue ratio from 10% to 5% on its own gives a modest hit-rate gain on workload-a, which supports the earlier cross-cycle suggestion that a smaller small queue is better. The per-op-time improvement is too big for a one-constant edit, so re-measure before crediting it to this change. The gain is also a single-trace result, so it may not generalize to other workloads.
