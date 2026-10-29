# Variant v003 Insight

## Helped

- per-op-time: The baseline ran SipHash four times per sketch call. v003 replaces this with one FxHash-style multiply-rotate hash plus a splitmix64 finish, and slices the four row indices out of that single 64-bit value. The map uses the same cheap hasher, so `get` now does two fast hashes where it used to do five SipHash passes, and median get and put latency both drop by about half.
- per-op-time (put tail): `put` at capacity now computes the incoming key's hash once and reuses it for both `increment_hashed` and `estimate_hashed`, and the victim is hashed only once. This removes most of the roughly fifteen SipHash passes on the admission-contest path, which is where the baseline's large put p99 came from.
- metadata-size: The per-cache `TinyLfuCache` struct got smaller because `BuildHasherDefault<FastHasher>` is zero-sized, while the default `RandomState` carries 16 bytes of SipHash keys. The headline per-node size and the sketch heap bytes are unchanged, as intended, because the width formula and depth were left alone.

## Hurt

- per-op-time (get tail): get p99 went up while get p50 halved. The likely cause is that the get path no longer has hashing to hide memory latency behind, so the four cache misses into the sketch rows and the map probe now set the tail. Switching to a flat `Vec<u32>` keeps the rows `width` apart and doesn't make them any closer together in memory. Benchmark noise could also explain this; it was not confirmed.

## Takeaway

Hashing, not data structure work, was the main cost of a baseline TinyLFU access. Hashing each key once with a cheap, well-mixed hash and splitting it into row indices roughly halves median latency, with hit-rate within noise and admission semantics unchanged. Since hashing no longer dominates, the remaining cost is scattered memory access across the four sketch rows. That makes cache-line-local sketch layouts, such as putting all of a key's counters in one 64-byte block, the natural next lever for per-op-time, especially the get tail.
