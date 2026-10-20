# Variant v003 Insight

## Helped

- per-op-time: Both get and put latency (median and tail) dropped well below the parent. A flat open-addressing table with linear probing keeps lookups in one or two adjacent cache lines. Hashing once per operation and reusing that hash for the sketch rows removes the second hash and the pointer-chasing map walk.
- metadata-size: The top-level `TinyLfuCache` struct shrank slightly, consistent with replacing the map handle with slab and table fields. `Node` stayed at its baseline size, as the hypothesis required.

## Hurt

- hit-rate: Nothing was hurt. It moved by a negligible amount, which is within what changing the hash source for the sketch rows could cause. The admission rule and LRU structure are unchanged, so this is not an algorithmic gain.

## Takeaway

Hash-table and hashing overhead dominated the baseline's per-op cost. Restructuring the storage (slab, flat table, single hash) cut latency substantially without changing cache policy or the `Node` footprint. The sketch (64 KiB) still dominates total metadata, so layout work like this will not move metadata-size. Only changing the sketch would.
