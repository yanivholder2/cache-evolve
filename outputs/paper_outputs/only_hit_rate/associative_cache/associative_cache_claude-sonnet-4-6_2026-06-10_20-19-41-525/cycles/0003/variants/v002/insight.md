# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Widening the cold threshold from `lfu_counter == 0` to `lfu_counter <= 1` caused items that had received exactly one genuine access to be evicted before they could be accessed again, removing entries from the hot set that the parent's stricter filter had correctly preserved.

## Takeaway

Raising the warmth threshold to require two accesses before shielding an item from eviction assumed that single-hit residency is scan noise, but on workload-a a single access is already a reliable signal of future reuse — making items with `lfu_counter == 1` eligible for eviction strips the cache of entries that would have hit. The parent's `counter == 0` boundary sits at the right point for this trace: the minimal "been accessed at all" criterion is sufficient to separate scan-once insertions from recurrently useful items.
