# Variant v002 Insight

## Hurt

- **per-op-time**: Tracking `last_access_epoch` on every get/put and reorganizing the bucket every 16 evictions added non-trivial overhead—incrementing epochs, updating item fields, and partitioning the vector in-place multiplied operation latency across all operations, undoing v003's early-exit gains.
- **hit-rate**: The reorganization logic disrupted the implicit ordering that the early-exit optimization relied on; clustering hot items at the front does not improve cache-hit rates if items are still searched linearly by key match rather than by access frequency.
- **metadata-size**: The new `last_access_epoch: usize` field increased the Item struct from 40 to 48 bytes, expanding the per-item footprint by 20%.

## Helped

(none)

## Takeaway

The attempt to boost CPU prefetch effectiveness through periodic access-based reorganization failed because the overhead of epoch tracking and vector reorganization exceeded any benefit. The reorganization strategy was fundamentally misaligned with the early-exit optimization: reordering by recency does not improve the success rate or speed of linear key searches. This variant demonstrates that micro-optimizations targeting spatial locality must directly serve the hot-path algorithm (as v003's early-exit did) rather than introducing secondary bookkeeping that adds cost without corresponding algorithmic benefit.
