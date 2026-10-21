# Variant v002 Insight

## Helped

- (none observed) The intended gain was in per-op-time (get), but this variant's fitness vector has no per-op-time measurement, so no improvement can be attributed to replacing the O(n) remove+push with a single swap.

## Hurt

- hit-rate: Promoting a hit item only one slot toward the protected end (transposition) instead of to the very back makes recency information decay slowly, so recently reused items stay near the eviction end and get evicted sooner than under true LRU.
- per-op-time / metadata-size: Both dimensions are missing from the fitness vector, so the variant cannot be compared with the parent on them; the hypothesis's main target went unverified.

## Takeaway

Swapping move-to-back for a one-step transposition cost hit rate on the LIRS workload-a trace, and the evaluation didn't report the per-op-time numbers the change was meant to improve. A speed-for-hit-rate trade like this can't be judged without the timing dimension, and the measured loss means the change needs a clear timing win to be worth keeping.
