# Variant v002 Insight

## Helped

- **per-op-time**: Replacing the O(ghost_size) `iter().any()` linear scan in `put()` with an O(1) `ghost_set.contains()` call cuts median `put` latency by nearly half and reduces tail latency measurably, with `get` also benefiting.

## Hurt

- **metadata-size**: Adding `ghost_set: HashSet<K>` grew the `S3Fifo` struct from 112 to 160 bytes; per-entry cost is unchanged, but every cache instance now carries 48 bytes of additional HashSet header overhead plus a heap-allocated backing array sized to `main_size`.

## Takeaway

The ghost-queue linear scan was the dominant cost in `put()` — replacing it with a HashSet membership oracle nearly halves median put latency with no algorithmic change to eviction policy or hit-rate. The overhead is a fixed per-instance struct cost that grows only with `main_size`, not with the number of stored entries, so it amortizes away at any practical cache capacity.
