# Variant v002 Insight

## Helped

- per-op-time: The `resident_set.contains()` O(1) guard inserted before both `position()` scans in `get()` and before the update-path scans in `put()` eliminates the double linear traversal on every cache miss and every new-key insertion, collapsing get p50 to zero and cutting put p50/p99 by roughly 7×.

## Hurt

- metadata-size: Adding `resident_set: HashSet<K>` pre-allocated at `capacity` increases the `S3Fifo` struct size by 48 bytes (224 → 272) because the HashSet carries its own internal allocation pointer, length, and capacity fields alongside the resident keys.

## Takeaway

The `resident_set` membership guard proves that miss-path scanning was the dominant fraction of per-op cost on this Zipfian workload: roughly 27% of accesses are misses, and each previously paid two full O(n) `position()` sweeps before returning `None`; new-key `put()` calls similarly scanned both deques before falling through to ghost-check and admission. Replacing those scans with a single hash lookup eliminates that work entirely. Hit rate is unaffected because the change touches only the lookup code paths — the eviction policy (small/main/ghost queues and frequency logic) is identical to the parent.
