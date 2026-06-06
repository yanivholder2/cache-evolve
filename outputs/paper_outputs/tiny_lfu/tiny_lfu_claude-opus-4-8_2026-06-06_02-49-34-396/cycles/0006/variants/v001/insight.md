# Variant v001 Insight

## Helped

- hit-rate: Flattening `Vec<Vec<u8>>` to a single row-major `Vec<u8>` indexed as `counts[row * width + pos]` is byte-for-byte semantics-preserving, so hit-rate held exactly where the parent left it — the refactor was clean, just not beneficial.

## Hurt

- per-op-time: Replacing the nested sketch with one contiguous buffer made `get` and `put` *slower* (both p50 and p99 rose), the opposite of the predicted prefetch win — removing per-row pointer indirection did nothing because the cost is the double-hash stride wandering across each 16 KB-wide row, which spills cache lines identically under either layout.

## Takeaway

This variant answers the standing cross-cycle question: the wide (factor-16) sketch's per-op tax is *not* a layout or heap-indirection artifact — flattening to a contiguous buffer left it worse, not better. The tax is the floor price of the 64 KB sketch's footprint and its within-row hash stride; recovering per-op-time will require shrinking or restructuring the sketch's access pattern, not just its allocation shape.
