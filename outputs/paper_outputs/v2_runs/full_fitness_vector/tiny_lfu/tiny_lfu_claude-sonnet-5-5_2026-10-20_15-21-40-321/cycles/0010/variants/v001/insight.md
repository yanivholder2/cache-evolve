# Variant v001 Insight

## Helped

- hit-rate: Effectively unchanged, which is what the hypothesis required. Admission (strict-greater, LRU-head victim), window sizing and aging were preserved, so swapping the map for a flat table did not change eviction decisions. The tiny drop is within the 0.001 tolerance.

## Hurt

- per-op-time: Both `get` and `put` p50/p99 got slower than the parent, so the expected latency win did not transfer. With cap=1024 the table and slab are small enough to sit in cache, so the pointer-based map was not the bottleneck. Linear probing, backward-shift deletion and the slab-index indirection add work that the saved allocation and pointer chasing did not offset. The `put` tail (p99) suffered most, consistent with shift-on-evict cost.
- metadata-size: `Node` grew from 24 to 32 bytes. Slab-index links and, most likely, a stored hash enlarge each node, which is the price of reusing one hash per op. The reported sketch heap also quadrupled, which suggests the sketch sizing was not kept the same as the parent's. That inflates memory without any hit-rate payoff. The top-level `TinyLfuCache` struct shrank slightly, but the flat table and slab storage sit outside that figure.

## Takeaway

The flat slab and linear-probing table from the baseline lineage did not transfer as a latency win to the v003 lineage. At this cache size the pointer map is already cheap, so the layout change mostly added per-node bytes and probe/shift overhead. A data-structure win measured on one lineage should be re-validated before it is assumed to compound with another. A single-hash reuse trick is worth keeping apart from the table swap if it is retried.
