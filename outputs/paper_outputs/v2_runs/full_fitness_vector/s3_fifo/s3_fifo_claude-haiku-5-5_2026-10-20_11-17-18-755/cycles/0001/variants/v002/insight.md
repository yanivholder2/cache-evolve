# Variant v002 Insight

## Helped

- per-op-time (put): The ghost-hit test on `put` moved from a linear `ghost.iter().any(...)` scan to an O(1) `ghost_index.contains(&key)` lookup, and that scan was the dominant cost on the miss path, so put latency falls at both p50 and p99.

## Hurt

- metadata-size: The new `ghost_index: HashSet<K>` field adds per-cache overhead to the `S3Fifo` struct itself, so the struct footprint grows even though the per-entry size is unchanged.

## Takeaway

Swapping the ghost membership test for a hash index is a pure data-structure change: hit-rate is identical to the parent, which confirms the eviction policy was left intact. The gain is confined to the put path, and the cost is a larger fixed struct footprint, with the ghost queue still paying O(n) on its own pop-back and position-based removal. The get-path latency shift is not explained by this diff, since `get()` and `find_entry_mut()` were not modified, so treat it as measurement noise or layout effect rather than a result of this change.
