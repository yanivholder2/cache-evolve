# Variant v002 Insight

Note: I could not list or diff the overlay source in this session (only Read and Write were usable). This insight rests on the hypothesis and the two fitness vectors, not on a line-level diff.

## Helped

- (none): No dimension improved. Hit-rate moved by a negligible amount in the wrong direction, and metadata-size stayed unchanged as the hypothesis required.

## Hurt

- per-op-time: Both get and put latency, at p50 and p99, got much worse. The added free-slot branch in the insert path does not explain that on its own, especially for get, which the hypothesis said it would not touch. This points to a code-layout or inlining side effect, or to a noisy run, rather than the cost of the branch.
- hit-rate: Admitting first touches unconditionally while a segment has free slots gave no gain, and the result is marginally below the parent's. So warm-up rejection by the second-hit filter was not costing hits on this trace.

## Takeaway

The hit-rate ceiling on this workload is not caused by warm-up loss. At a 2048-entry capacity the cache fills quickly, so the filter spends almost all of its time in the full-segment regime where the change does nothing. A small conditional on the put path should not slow get, so a latency regression of this size on get is more likely a measurement or compilation artifact than a real cost of the idea. The variant should be re-measured before the idea is judged on latency.
