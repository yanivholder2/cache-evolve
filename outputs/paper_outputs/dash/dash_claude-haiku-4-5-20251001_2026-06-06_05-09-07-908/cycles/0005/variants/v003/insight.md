# Variant v003 Insight

## Helped

- per-op-time (get): Adaptive threshold computed once at initialization and passed to promotion logic eliminates per-get lookups, and well-calibrated gating prevents unnecessary promotion cascades for this geometry.
- per-op-time (put): Refined promotion selectivity reduces eviction cascades that would otherwise demote items through multiple tiers on put.

## Hurt

(none)

## Takeaway

The adaptive frequency-gating threshold scaled to stash bucket count (formula: max(1, stash_buckets / 4)) succeeds on this baseline geometry: hit rate remains stable at 0.6634, while get latency drops 42–43% (p50/p99) and put latency improves 9–19% across percentiles. By moving threshold computation from a hardcoded constant to a geometry-aware initialization parameter, v003 demonstrates that adaptive gating can unlock efficiency gains without sacrificing hit quality — a key enabler for composition across varying allocation profiles.
