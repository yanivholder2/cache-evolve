# Variant v003 Insight

## Helped

- per-op-time: `get` is now a single FxHash lookup in `index` plus a slab access. Before, it walked the `small`→`main` chain one entry at a time, so its cost no longer grows with capacity. That is why `get` p50 and p99 both dropped sharply.
- per-op-time (`put`): `put` sped up even more than `get`, although the hypothesis only targeted `get`. The parent paid for a full two-queue residency scan, a ghost `iter().any`, and a `retain` on every miss. All three are now hashed O(1) operations. The stamp-based lazy deletion keeps `retain` only as a rare compaction once the ghost deque exceeds 2×`ghost_cap`, and that change also flattened the `put` tail.

## Hurt

- metadata-size: The summary did not move, but only because it measures `Entry<K,V>`, which was kept unchanged on purpose. The real per-resident cost went up: each entry now sits in an `Option<Entry>` slab slot, has a `u32` in a queue, and has a `(K, u32)` record in `index`. Each ghost now costs a `(K, u64)` stamped record (with up to twice as many physical records) plus a `ghost_set` entry, where the parent stored a bare `K`. The fixed `S3Fifo` struct row also grew because of the extra maps, vectors, and counters. The dimension as defined cannot see any of this cost.

## Takeaway

Hit-rate came out identical to the parent, which confirms the refactor kept every decision the same. The stamped ghost lazy-deletion scheme matches the parent's `retain` + `pop_back` behaviour exactly, because the parent's ghost could never hold duplicate keys. So this variant cleanly re-measures the adaptive-split policy. The parent's latency numbers were dominated by its O(capacity) scans, and that is why its earlier "unexplained" latency gain should not be credited to the policy. From now on, hashed indexing should be the baseline for per-op-time comparisons. `metadata-size` should also be extended to count index, slab, and ghost overhead, so that structural changes like this one show their memory cost.
