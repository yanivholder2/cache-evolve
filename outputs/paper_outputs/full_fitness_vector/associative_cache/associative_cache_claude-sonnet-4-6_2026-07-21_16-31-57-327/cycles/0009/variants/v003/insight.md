# Variant v003 Insight

## Helped

_(none)_

## Hurt

- **hit-rate**: Resetting `lfu_counter` on the item displaced to `position` after a deep-cold promotion strips the warm flag from items that may have legitimately earned it — the item at `position - 1` is not necessarily stale, and stripping its flag on a rare deep-cold hit converts a genuinely warm item into an immediate cold eviction target, increasing churn on the Zipfian workload rather than reducing it.
- **per-op-time**: The conditional write (`items[position].lfu_counter = 0`) after the swap loop introduces an extra store on the cold-hit path, raising put p50 and both p99 tail latencies even though this path fires only for accesses from the two deepest positions.

## Takeaway

The hypothesis assumed that any item sitting one step above the two deepest positions is a "stale" warm-marker holder, but on a Zipfian workload those items are legitimately warm: they earned their flag through a real access and only drifted near the tail due to low access frequency. Clearing the flag on displacement therefore evicts items that still have residual value, and the net effect is a lower hit rate. The flag-expiry mechanism needs a stronger signal than "the promoted item started near the tail" to correctly identify genuinely stale warm markers.
