# Variant v003 Insight

## Helped

- hit-rate: Raising `protected_cap` from `max_size/2` to `3*max_size/4` keeps reused items in the protected segment against probationary scan churn slightly longer, nudging hit-rate up a hair over the parent.

## Hurt

- per-op-time: The deeper protected segment lengthens protected-MRU rotations, surfacing as a doubling of `get` p99 tail latency even though the p50 summary stayed pinned at the floor.

## Takeaway

Between the two falsified walls (`max_size/2` demotes winners, `max_size-1` collapses to LRU), the mid-range `3*max_size/4` cap moves hit-rate only marginally — this knob is a nearly flat lever in the reused-item range, and its main measurable cost is `get` tail latency from longer rotations rather than any p50 regression.
