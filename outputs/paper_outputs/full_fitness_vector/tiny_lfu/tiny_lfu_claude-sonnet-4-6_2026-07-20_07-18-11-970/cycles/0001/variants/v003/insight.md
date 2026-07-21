# Variant v003 Insight

## Helped

- hit-rate: The `h1 + i·h2 mod width` linear-combination formula produces sufficiently independent row positions to preserve sketch accuracy nearly unchanged from baseline.

## Hurt

- per-op-time: Replacing four independent key hashes with a chained double-hash (`h2` is derived by hashing `h1`) introduces a serial data dependency that prevents the CPU from pipelining the row-position computations, and the added `wrapping_mul` per row compounds the overhead — the net result is slower, not faster, than four independent `DefaultHasher` calls on a cheap integer key.

## Takeaway

The double-hash trick trades hash count for arithmetic, which pays off when keys are expensive to hash (large strings, structs), but backfires on `u64` keys where each `DefaultHasher` invocation is already near-zero cost and the serial `h1 → h2` dependency destroys instruction-level parallelism. The hypothesis assumed all four per-row hashes in the baseline were the bottleneck; for this key type the chained dependency and per-row multiply imposed by `row_pos` cost more than the two hashes saved.
