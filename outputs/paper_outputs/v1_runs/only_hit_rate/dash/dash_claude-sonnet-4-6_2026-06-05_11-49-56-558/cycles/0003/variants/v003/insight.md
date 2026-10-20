# Variant v003 Insight

## Helped

## Hurt

- hit-rate: The threshold gate prevents co-resident decay until the hit item accumulates ≥ 4 hits, but this slow ramp-up delays frequency discrimination long enough that counter-based eviction still cannot match ClassicLRU's strict positional recency ordering on this workload.

## Takeaway

The threshold gate did soften the collateral drain that made the prior cycle's blanket-decay `AgingLfu` worse than baseline, but it was not sufficient to overcome the fundamental cost of abandoning position-based recency. On workload-a, item position encodes a stronger future-access signal than a thresholded frequency counter: newly inserted items with counter zero survive eviction pressure until a hot item crosses the threshold, while ClassicLRU would have already displaced them via recency promotion. Threshold-gating decay rescues co-residents from aggressive aging but introduces its own blind spot — sluggish counter discrimination in the early life of every item.
