# Variant v003 Insight

## Helped

- **per-op-time**: The 75/25 geometry allocates 5% more capacity to the stash (25% vs 20%), reducing lookup depth and insertion contention; the more restrictive divisor formula (÷4 instead of ÷3) gates promotion more stringently, preventing excessive tier migration that triggers cascading evictions and slowdowns.

## Hurt

(none)

## Takeaway

The adaptive divisor formula's optimal divisor scales with available stash capacity. v003 demonstrates that with increased stash allocation (75/25 vs v002's 80/20), a larger divisor (4 vs 3) better controls promotion selectivity—improving latency across all operations while preserving hit-rate. This suggests the formula is geometry-aware: more stash headroom permits stricter promotion thresholds without sacrificing retention, confirming that threshold tuning and geometry are not fully orthogonal but can be co-optimized.
