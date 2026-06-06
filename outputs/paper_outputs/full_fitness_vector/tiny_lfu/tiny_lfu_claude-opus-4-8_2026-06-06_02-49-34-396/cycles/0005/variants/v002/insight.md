# Variant v002 Insight

## Helped

- hit-rate: Raising `SKETCH_DEPTH` from 4 to 6 adds two more independent double-hash rows, so the `min()` reduction in `estimate` is taken over more samples and the count-min overcount tightens, lifting admission accuracy and hit-rate.

## Hurt

- per-op-time: `increment` and `estimate` now loop over 6 rows instead of 4, touching 50% more sketch cells per op; both `get` and `put` p50 and p99 rise modestly from the extra row scans and cache traffic.
- metadata-size: The tracked `Node` summary stays at 24 (node layout untouched), but the `CountMinSketch` heap allocation grows from 16 KiB to 24 KiB as the two added rows widen the backing `vec![vec![0u8; width]; depth]`.

## Takeaway

Sketch depth is still a live precision lever, not saturated at 4: two more hash rows buy a small hit-rate gain orthogonal to width by reducing the min-estimate overcount. But the gain is bought with proportional per-op cost (6 vs 4 row scans) and 50% more sketch heap, so depth trades latency and sketch memory for accuracy rather than being free.
