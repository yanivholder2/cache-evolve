# Variant v002 Insight

## Helped

- hit-rate: Reducing the small queue from `capacity/20` to `capacity/40` expands main residency to ~97.5% of capacity, allowing more of the Zipfian hot working set to coexist in the resident queue simultaneously and continuing the monotone hit-rate improvement seen across prior halving steps.

## Hurt

- per-op-time: The larger main queue (from 1946 to 1997 slots at capacity=2048) lengthens the linear `position()` scan in both `get()` and `put()` for resident keys — since most hits resolve in main, even a modest increase in main depth lifts get p99 and substantially raises put tail latency.

## Takeaway

Halving the small-queue fraction from 1/20 to 1/40 continued to nudge hit rate upward, confirming that main still benefits from the extra residency room. However, the change revealed a latency trade-off: every slot gained by main is also a slot the linear key-scan must traverse on a hit, and at this scale (capacity=2048, main ≈ 1997 entries) the longer main queue visibly degrades write and tail-read latency even though the hit-rate gain persists. Further shrinking the small queue risks amplifying this scan cost faster than the hit-rate benefit grows.
