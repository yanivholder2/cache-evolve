# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Moving the rejected victim to the MRU tail changes which resident is the head on the next miss, so the next admission decision is made against a different, typically hotter, resident; on the lirs workload-a trace this lowered the hit rate rather than letting frequency estimates win admission.
- per-op-time: The rejected-admission path now performs an extra unlink and push_back on every rejected put, which is the added work the hypothesis accepted as a cost; the put latency increase is attributable to this branch.

## Takeaway

Rotating a rejected victim to the tail does not free the cache from a "stale" LRU resident: it only swaps which resident is compared against the next newcomer, and on this trace the swapped-in resident is not a worse victim. The frequency-based admission gate was already doing the protective job, so recency rotation on rejection adds cost without improving admission quality.
