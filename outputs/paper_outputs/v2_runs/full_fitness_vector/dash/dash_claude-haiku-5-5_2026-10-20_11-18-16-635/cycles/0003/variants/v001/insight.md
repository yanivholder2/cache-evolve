# Variant v001 Insight

## Helped

(none measured)

## Hurt

- per-op-time: Breaking equal-counter victim ties by LRU age in `evict_one()` adds recency work on the eviction path, and both get and put slowed relative to the parent even though the hypothesis left the get path unchanged; the extra cost is not offset by any hit-rate gain.

## Takeaway

On workload-a, resolving counter ties by recency left hit rate exactly where the parent put it, so the counter ranking itself, not the order among tied slots, is what carries v003's hit-rate effect on this trace. Any future tie-break work should be justified by a trace where ties actually decide evictions, and should be checked against per-op cost before it is kept.
