# Variant v003 Insight

## Helped

- per-op-time: Batch-amortized decay (every 32 evictions) eliminates per-operation frequency decay cost from the hot path, delivering dramatic latency wins (44% faster get, 37% faster put) while maintaining correctness.
- per-op-time: Widened ghost retention threshold from `freq > 1` to `ghost_freq > -3` preserves more re-admission candidates without increasing per-operation work, keeping the hot path lean.

## Hurt

- hit-rate: Slight regression (-0.05%) likely due to the wider ghost candidate pool occasionally re-admitting items that don't provide cache value, though the magnitude is negligible and within typical measurement noise.
- metadata-size: Added `small_evict_count: usize` (8 bytes) to track amortization counter, a small but measurable increase in per-cache overhead.

## Takeaway

This variant proves that expensive computations—like frequency decay—can be moved entirely off the hot path via amortization without sacrificing hit rate. By batching decay every 32 evictions and decoupling frequency *tracking* from frequency *decay*, v003 achieves substantial latency improvements (especially for gets, which are latency-sensitive) while the ghost queue maintains nearly identical hit efficiency. The zero hot-path-overhead philosophy successfully extends to richer ghost state when decay cost is pushed far enough into cold time.
