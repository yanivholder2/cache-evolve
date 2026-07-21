# Variant v003 Insight

## Helped

- hit-rate: The wider second-chance window (up to five decrements before eviction instead of three) gave the Zipfian hot set marginally stronger protection in the main queue, producing a small but positive hit-rate gain on the workload-a trace.

## Hurt

- per-op-time: `evict_main()` must now traverse and decrement through up to five non-zero-freq entries per eviction event rather than three, increasing the average loop iterations per `put()` that triggers eviction and raising both put p50/p99 and, to a lesser extent, get p50.

## Takeaway

Raising `MAX_FREQ` from 3 to 5 confirms that the hot-set protection window was already near its effective ceiling at the parent value: the hit-rate moved only slightly upward, indicating the Zipfian hot set was already well-retained. The tradeoff is not free — `evict_main()`'s unbounded second-chance loop does proportionally more work per eviction cycle, and the per-op latency cost is larger in relative terms than the hit-rate benefit. Future changes that want to improve hit-rate on Zipfian workloads should look beyond `MAX_FREQ` tuning.
