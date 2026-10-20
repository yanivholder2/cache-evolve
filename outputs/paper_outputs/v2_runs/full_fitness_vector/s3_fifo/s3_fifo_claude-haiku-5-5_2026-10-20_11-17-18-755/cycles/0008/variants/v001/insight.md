# Variant v001 Insight

## Helped

- per-op-time: The id-keyed `index` replaces the two linear `position` scans on the resident path with one hash lookup followed by `queue.get_mut(head - id)`, which is the direct cause of the much lower get latency, both at p50 and at the tail.
- per-op-time: Put on a resident key now writes through the same index slot instead of scanning both deques, so the put path also loses its linear search and gets faster at p50 and p99.

## Hurt

- metadata-size: The `S3Fifo` container gained a `HashMap<K, Slot>` plus `small_head` and `main_head` counters, so the container footprint grew even though the per-entry `Entry` size is unchanged. The summary field reports only `Entry` and therefore hides this cost.

## Takeaway

Keying residents by a stable id (`head - position`) makes the resident path a constant-time lookup without changing any eviction decision, which is why the hit rate is identical to the parent. The price is fixed per-container state for the index and head counters, and the summary metadata number does not show it, so the container-level figure is the one to track when judging this change.
