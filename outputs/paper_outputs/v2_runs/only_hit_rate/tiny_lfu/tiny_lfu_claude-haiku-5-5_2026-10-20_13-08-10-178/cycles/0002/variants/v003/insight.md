# Variant v003 Insight

## Helped

## Hurt

## Takeaway

Flattening the sketch's `Vec<Vec<u32>>` into one `Vec<u32>` indexed as `row * width + pos` is a pure layout change: each (row, position) pair still maps to the same logical counter, so hit-rate is unchanged, and it is. The fitness vector carries only hit-rate, so the per-op-time gain the hypothesis predicted (fewer allocations and one less pointer hop per row) is not measured here and remains unverified. Any future per-op-time dimension is the only way to confirm this layout change paid off.
