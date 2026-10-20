# Variant v002 Insight

## Helped

- get: The per-queue id scheme (`index = head - id`) replaces the linear scan over `small` and `main` with one hash lookup and an indexed access, so hits no longer pay for a walk through the queues.
- put: Resident-key lookup now goes through the two indexes instead of `find_entry_mut`, which scanned both queues on every put, including misses. The extra index insert on each push costs less than the scan it removes.

## Hurt

- metadata-size: `S3Fifo` gains two `HashMap` indexes and two head counters, so the container is larger. `Entry` is untouched, so the per-entry footprint holds and the cost is fixed container overhead, not growth per cached item.

## Takeaway

The index change does what H02 targeted: lookups stop scanning the queues, and hit rate is unchanged because eviction, ghost, and promotion logic are untouched. The price is fixed container state rather than per-entry bytes, and the put-side hashing the hypothesis expected to add cost was more than offset by removing the resident-key scan.
