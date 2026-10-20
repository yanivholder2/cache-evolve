# Variant v001 Insight

## Helped

- per-op-time: replacing the linear `ghost.iter().any(...)` membership scan in `put` with an O(1) `ghost_set.contains(&key)` cut put p50 and get p50, since every put miss (and the hot path generally) no longer walks the full ghost deque.

## Hurt

- per-op-time: put p99 rose — the `HashSet` insert/remove on every ghost `push_front`/`pop_back` plus occasional hash resize concentrate cost into the tail, trading a smaller, predictable per-element scan for rarer but larger hashing/rehash spikes.

## Takeaway

The O(1) ghost index pays off at the median exactly where v003's fuller ghost hurt most — typical put/get latency drops — and hit-rate stays bit-for-bit identical because `ghost_set` mirrors `ghost` without changing membership semantics. The catch is a worse put tail: maintaining a parallel hash structure shifts the linear scan's cost into intermittent hashing spikes, and the `S3Fifo` container grows (unscored, since `Entry` stays 24 bytes).
