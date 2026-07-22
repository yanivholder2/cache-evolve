# Variant v001 Insight

## Hurt

- **per-op-time**: Changing geometry from 32×64 to 64×32 (halving bucket width from 64 to 32 items) caused severe latency regression—get p99 doubled from 100 ns to 200 ns, put p50 tripled from 100 ns to 300 ns, and put p99 doubled from 200 ns to 400 ns. The early-exit optimization loses its effectiveness when buckets are narrower because the cost-benefit of scanning fewer items diminishes at smaller bucket sizes.
- **hit-rate**: Cache behavior shifted slightly as a side effect of geometry change (0.664 → 0.6635), showing that bucket count influences eviction dynamics even at the same total capacity.

## Takeaway

The micro-optimizations in v003 (early-exit on key match and cache-line alignment) are inseparable from the 32×64 bucket geometry on which they were tuned. By naively scaling the geometry factor (halving instead of quartering the bucket count), this variant demonstrates that wide buckets are not merely a nice-to-have for hit-rate—they are essential for the latency-efficiency gains to materialize. The optimization strategy is geometry-dependent: narrow buckets (64×32) forfeit the latency benefits that wide buckets (32×64) gained, confirming that v003's success is a localized tuning optimum rather than a generalizable principle for all geometries.
