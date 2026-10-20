# Variant v003 Insight

## Helped

- per-op-time: The ghost queue now holds 64-bit fingerprints tracked by a `HashSet`, so the `put` miss path replaces the linear `ghost.iter().any(...)` scan and `retain` with O(1) hash lookups, which is the likely source of the drop in put tail latency.
- per-op-time: Eviction pushes `fingerprint(&tail.key)` instead of cloning `tail.key` into the ghost, removing a key clone from every small-queue eviction on the put path.

## Hurt

- metadata-size: The ghost now carries three structures (the fingerprint `VecDeque`, `ghost_set`, and `ghost_copies`) plus stale queue copies until compaction, so the `S3Fifo` container grew even though `Entry` stayed at 16 bytes.

## Takeaway

The change keeps the frequency side-array and promotion rules intact, so the hit-rate being unchanged is expected, and the gain is almost entirely on the put side from replacing the linear ghost scan with a hash set. The get path code is byte-identical to the parent, so the get-latency movement is not something this diff explains and should not be credited to the ghost change. The container-size cost is the price of the fingerprint ghost, and any future version should record the whole `S3Fifo` footprint, not `Entry` alone.
