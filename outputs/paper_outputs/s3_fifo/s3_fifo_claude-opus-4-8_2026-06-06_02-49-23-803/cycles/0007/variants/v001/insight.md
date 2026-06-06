# Variant v001 Insight

## Helped

- per-op-time: replacing the `main.iter().chain(small)` linear scan in `get`/`find_entry_mut` with a single `entries.get_mut(key)` probe (backed by a HashMap parameterised with an inline FNV/multiplicative `FastHasher`) makes lookups structurally O(1), breaking through v002's scan-order-bound get-p50 floor — and the same constant-time resolve also drops put's update-in-place path, cutting put p50/p99 and get p99 since misses and tail cases no longer walk a queue.

## Hurt

- metadata-size: unchanged on the scored summary — `Entry<K,V>` keeps its `{ key, value, freq }` fields at 24 bytes — but the `S3Fifo` struct itself grew (the added `HashMap` field) since each resident key now lives in both a key-only queue and the `entries` map; this container/per-key duplication is currently unscored.

## Takeaway

Moving the resident store to a key-only FIFO queue plus an O(1) hash index is the direct structural attack on get-p50: it converts the median linear scan into one hash probe and beats SipHash with an inline integer hasher, while mirroring every eviction decision through `entries` keeps hit-rate bit-identical at 0.7312. Unlike v002's scan-reorder — which only shifted cost to the tail — this lowers median *and* tail latencies together, at the cost of storing each key twice (a latent memory trade-off the metadata summary doesn't yet capture).
