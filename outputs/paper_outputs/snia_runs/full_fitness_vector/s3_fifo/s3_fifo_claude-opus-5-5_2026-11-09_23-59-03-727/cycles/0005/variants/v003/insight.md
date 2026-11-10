# Variant v003 Insight

## Helped

(none: hit-rate is bit-identical at all four trace/size points, as expected. The change only pre-sizes `index` and `ghost_index` in `try_new`, and every policy path (`get`, `put`, `push_ghost`, `evict_main`, `evict_small`) is untouched. The per-op-time summary (get p50) and put p50 did not move either.)

## Hurt

- per-op-time: The `put` p99 got clearly worse, the opposite of what the hypothesis predicted, and `get` p99 slipped slightly. hashbrown already rounds a `with_capacity(n)` request up to a power-of-two bucket count above `n·8/7`. At the measured sizes (16384 and 131072) the parent's resident index already sat at about 50% load. Asking for `2·capacity` doubles the bucket and control-byte arrays without lowering probe length in any useful way. The bigger, sparser tables spread each lookup's cache lines over twice the memory, so the miss-heavy `put` path (`index.get` → `ghost_index.remove` → `index.insert`, plus the ghost-index work in `push_ghost`) takes more LLC/TLB misses in the tail.
- metadata-size: The metric did not move, because `Entry` stays 24 bytes and the `S3Fifo` header is unchanged. But this hides a real cost: both hash tables now use about twice the heap, which is exactly the trade-off the hypothesis said the metric cannot see.

## Takeaway

On this Fx-hashed slab, hash-table occupancy is not the bottleneck. hashbrown's power-of-two rounding already leaves the resident index about half empty at the benchmarked capacities, and the ghost index is the only table that ran near 75% load. Extra headroom makes timing worse because memory footprint and locality, not probe length, drive the `put` tail. So the cycle-4 lesson ("lower load helps tails") does not reverse cleanly. Future layout work should shrink the tables or make them denser, for example a tighter ghost index or smaller value types in the index, rather than over-allocating them.
