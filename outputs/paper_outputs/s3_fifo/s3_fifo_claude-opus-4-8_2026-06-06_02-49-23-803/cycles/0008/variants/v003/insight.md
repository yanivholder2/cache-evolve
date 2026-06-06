# Variant v003 Insight

## Helped

- per-op-time: replacing put's linear `ghost.iter().any(...)` scan with a single O(1) `matches!(entries.get(&key), Some(Slot::Ghost))` probe over the map `put` already touches on its miss path collapses both put p50 and put p99 — the ghost-membership test no longer walks the order queue.

## Hurt

- per-op-time: get p50 ticked up and get p99 worsened because every lookup now matches on the `Slot` enum (a discriminant check distinguishing `Resident` from `Ghost`) and the `entries` map carries tombstones alongside live entries, enlarging the table the hot `get` probe walks.
- metadata-size: the scored `Entry<u64,u64>` summary is unchanged at 24 (the `Slot` enum lives in the container value type, not in `Entry`), but the `S3Fifo` struct grew by the added `ghost_count: usize` field — currently unscored container overhead.

## Takeaway

Folding ghost membership into the existing `entries` map as value-less `Slot::Ghost` tombstones eliminates put's linear ghost scan without adding a second hash table, trading a small, constant get-path tax (one enum-discriminant match plus a tombstone-inflated map) for a large put-latency win. The tombstone-plus-stale-order-queue design keeps hit-rate bit-identical, confirming the linear ghost scan — not the hash index — was the binding put cost.
